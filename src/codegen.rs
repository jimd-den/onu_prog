use crate::mir::{MirProgram, MirFunction, MirInstruction, MirOperand, MirLiteral, MirBinOp, MirTerminator};
use crate::types::OnuType;
use inkwell::context::Context;
use inkwell::builder::Builder;
use inkwell::module::Module;
use inkwell::values::{FunctionValue, BasicValueEnum, BasicValue, PointerValue};
use inkwell::types::{BasicTypeEnum, BasicType, BasicMetadataTypeEnum};
use inkwell::passes::PassManager;
use std::collections::HashMap;

pub trait CodeGenerator {
    fn generate(&self, program: &MirProgram) -> Result<Vec<u8>, String>;
}

pub struct LlvmGenerator<'ctx> {
    context: &'ctx Context,
    module: Module<'ctx>,
    builder: Builder<'ctx>,
    registry: Option<crate::registry::Registry>,
}

impl<'ctx> LlvmGenerator<'ctx> {
    pub fn get_ir_string(&self) -> String {
        self.module.print_to_string().to_string()
    }

    pub fn new(context: &'ctx Context, module_name: &str, registry: Option<crate::registry::Registry>) -> Self {
        let module = context.create_module(module_name);
        let builder = context.create_builder();
        Self { context, module, builder, registry }
    }

    fn run_optimizations(&self) {
        let fpm = PassManager::create(&self.module);

        fpm.add_instruction_combining_pass();
        fpm.add_reassociate_pass();
        fpm.add_gvn_pass();
        fpm.add_cfg_simplification_pass();
        fpm.add_promote_memory_to_register_pass(); // Essential for mem-based SSA

        fpm.initialize();

        for function in self.module.get_functions() {
            fpm.run_on(&function);
        }
    }

    fn onu_type_to_llvm(&self, typ: &OnuType) -> BasicTypeEnum<'ctx> {
        match typ {
            OnuType::I64 => self.context.i64_type().as_basic_type_enum(),
            OnuType::F64 => self.context.f64_type().as_basic_type_enum(),
            OnuType::Boolean => self.context.bool_type().as_basic_type_enum(),
            OnuType::Strings => self.context.i8_type().ptr_type(inkwell::AddressSpace::default()).as_basic_type_enum(),
            OnuType::Tuple(types) => {
                let llvm_types: Vec<BasicTypeEnum> = types.iter().map(|t| self.onu_type_to_llvm(t)).collect();
                self.context.struct_type(&llvm_types, false).as_basic_type_enum()
            }
            _ => self.context.i64_type().as_basic_type_enum(),
        }
    }

    fn generate_function(&self, mir_func: &MirFunction) -> Result<Option<FunctionValue<'ctx>>, String> {
        if mir_func.args.iter().any(|arg| matches!(arg.typ, OnuType::Shape(_) | OnuType::Nothing)) {
            return Ok(None);
        }

        let fn_name = if mir_func.name == "main" || mir_func.name == "run" { "main" } else { &mir_func.name };
        let function = self.module.get_function(fn_name).unwrap();
        
        let mut ssa_storage: HashMap<usize, (PointerValue<'ctx>, BasicTypeEnum<'ctx>)> = HashMap::new();
        
        let entry_bb = self.context.append_basic_block(function, "entry");
        self.builder.position_at_end(entry_bb);

        let mut var_types: HashMap<usize, BasicTypeEnum<'ctx>> = HashMap::new();
        for arg in &mir_func.args {
            var_types.insert(arg.ssa_var, self.onu_type_to_llvm(&arg.typ));
        }
        
        // Pass to determine types
        for block in &mir_func.blocks {
            for inst in &block.instructions {
                match inst {
                    MirInstruction::Assign { dest, src } => {
                        let typ = match src {
                            MirOperand::Constant(lit) => match lit {
                                MirLiteral::I64(_) => self.context.i64_type().as_basic_type_enum(),
                                MirLiteral::F64(_) => self.context.f64_type().as_basic_type_enum(),
                                MirLiteral::Boolean(_) => self.context.bool_type().as_basic_type_enum(),
                                MirLiteral::Text(_) => self.context.i8_type().ptr_type(inkwell::AddressSpace::default()).as_basic_type_enum(),
                                MirLiteral::Nothing => self.context.i64_type().as_basic_type_enum(),
                            },
                            MirOperand::Variable(id) => *var_types.get(id).unwrap_or(&self.context.i64_type().as_basic_type_enum()),
                        };
                        var_types.insert(*dest, typ);
                    }
                    MirInstruction::BinaryOperation { dest, op, lhs, .. } => {
                        let typ = match op {
                            MirBinOp::Eq | MirBinOp::Gt | MirBinOp::Lt => self.context.i64_type().as_basic_type_enum(),
                            _ => match lhs {
                                MirOperand::Variable(id) => *var_types.get(id).unwrap_or(&self.context.i64_type().as_basic_type_enum()),
                                _ => self.context.i64_type().as_basic_type_enum(),
                            }
                        };
                        var_types.insert(*dest, typ);
                    }
                    MirInstruction::Call { dest, name, .. } => {
                        let ret_type = if let Some(f) = self.module.get_function(name) {
                            f.get_type().get_return_type().unwrap_or(self.context.i64_type().as_basic_type_enum())
                        } else if name == "broadcasts" || name == "emit" {
                            self.context.i32_type().as_basic_type_enum()
                        } else {
                            let actual_name = if let Some(idx) = name.find('_') { &name[..idx] } else { name };
                            if let Some(sig) = self.registry.as_ref().and_then(|r| r.get_signature(actual_name)) {
                                self.onu_type_to_llvm(&sig.return_type)
                            } else {
                                self.context.i64_type().as_basic_type_enum()
                            }
                        };
                        var_types.insert(*dest, ret_type);
                    }
                    MirInstruction::Tuple { dest, elements } => {
                        let mut elem_types = Vec::new();
                        for e in elements {
                            match e {
                                MirOperand::Constant(lit) => match lit {
                                    MirLiteral::I64(_) => elem_types.push(self.context.i64_type().as_basic_type_enum()),
                                    MirLiteral::F64(_) => elem_types.push(self.context.f64_type().as_basic_type_enum()),
                                    MirLiteral::Boolean(_) => elem_types.push(self.context.bool_type().as_basic_type_enum()),
                                    MirLiteral::Text(_) => elem_types.push(self.context.i8_type().ptr_type(inkwell::AddressSpace::default()).as_basic_type_enum()),
                                    MirLiteral::Nothing => elem_types.push(self.context.i64_type().as_basic_type_enum()),
                                },
                                MirOperand::Variable(id) => elem_types.push(*var_types.get(id).unwrap_or(&self.context.i64_type().as_basic_type_enum())),
                            }
                        }
                        let struct_type = self.context.struct_type(&elem_types, false);
                        var_types.insert(*dest, struct_type.as_basic_type_enum());
                    }
                    MirInstruction::Index { dest, subject, index } => {
                        if let MirOperand::Variable(id) = subject {
                            if let Some(BasicTypeEnum::StructType(st)) = var_types.get(id) {
                                let field_type = st.get_field_type_at_index(*index as u32).unwrap();
                                var_types.insert(*dest, field_type);
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        for (id, typ) in &var_types {
            let ptr = self.builder.build_alloca(*typ, &format!("v{}", id)).unwrap();
            ssa_storage.insert(*id, (ptr, *typ));
        }

        for (i, arg) in function.get_param_iter().enumerate() {
            let mir_arg = &mir_func.args[i];
            let (ptr, _) = ssa_storage.get(&mir_arg.ssa_var).unwrap();
            self.builder.build_store(*ptr, arg).unwrap();
        }

        let mut llvm_blocks = HashMap::new();
        for mir_block in &mir_func.blocks {
            let llvm_block = self.context.append_basic_block(function, &format!("bb{}", mir_block.id));
            llvm_blocks.insert(mir_block.id, llvm_block);
        }

        if let Some(first_block) = mir_func.blocks.first() {
            let target = llvm_blocks.get(&first_block.id).unwrap();
            self.builder.build_unconditional_branch(*target).unwrap();
        }

        for mir_block in &mir_func.blocks {
            let llvm_block = llvm_blocks.get(&mir_block.id).unwrap();
            self.builder.position_at_end(*llvm_block);

            for inst in &mir_block.instructions {
                match inst {
                    MirInstruction::Assign { dest, src } => {
                        let val = self.operand_to_llvm(src, &ssa_storage)?;
                        let (ptr, _) = ssa_storage.get(dest).unwrap();
                        self.builder.build_store(*ptr, val).unwrap();
                    }
                    MirInstruction::BinaryOperation { dest, op, lhs, rhs } => {
                        let l_val = self.operand_to_llvm(lhs, &ssa_storage)?;
                        let r_val = self.operand_to_llvm(rhs, &ssa_storage)?;
                        let res = match op {
                            MirBinOp::Add | MirBinOp::Sub | MirBinOp::Mul | MirBinOp::Div => {
                                if l_val.is_int_value() {
                                    match op {
                                        MirBinOp::Add => self.builder.build_int_add(l_val.into_int_value(), r_val.into_int_value(), "addtmp"),
                                        MirBinOp::Sub => self.builder.build_int_sub(l_val.into_int_value(), r_val.into_int_value(), "subtmp"),
                                        MirBinOp::Mul => self.builder.build_int_mul(l_val.into_int_value(), r_val.into_int_value(), "multmp"),
                                        MirBinOp::Div => self.builder.build_int_signed_div(l_val.into_int_value(), r_val.into_int_value(), "divtmp"),
                                        _ => unreachable!(),
                                    }.unwrap().as_basic_value_enum()
                                } else {
                                    match op {
                                        MirBinOp::Add => self.builder.build_float_add(l_val.into_float_value(), r_val.into_float_value(), "addtmp"),
                                        MirBinOp::Sub => self.builder.build_float_sub(l_val.into_float_value(), r_val.into_float_value(), "subtmp"),
                                        MirBinOp::Mul => self.builder.build_float_mul(l_val.into_float_value(), r_val.into_float_value(), "multmp"),
                                        MirBinOp::Div => self.builder.build_float_div(l_val.into_float_value(), r_val.into_float_value(), "divtmp"),
                                        _ => unreachable!(),
                                    }.unwrap().as_basic_value_enum()
                                }
                            }
                            MirBinOp::Eq | MirBinOp::Gt | MirBinOp::Lt => {
                                let cond = match op {
                                    MirBinOp::Eq => if l_val.is_int_value() { self.builder.build_int_compare(inkwell::IntPredicate::EQ, l_val.into_int_value(), r_val.into_int_value(), "eqtmp") } else { self.builder.build_float_compare(inkwell::FloatPredicate::OEQ, l_val.into_float_value(), r_val.into_float_value(), "eqtmp") },
                                    MirBinOp::Gt => if l_val.is_int_value() { self.builder.build_int_compare(inkwell::IntPredicate::SGT, l_val.into_int_value(), r_val.into_int_value(), "gttmp") } else { self.builder.build_float_compare(inkwell::FloatPredicate::OGT, l_val.into_float_value(), r_val.into_float_value(), "gttmp") },
                                    MirBinOp::Lt => if l_val.is_int_value() { self.builder.build_int_compare(inkwell::IntPredicate::SLT, l_val.into_int_value(), r_val.into_int_value(), "lttmp") } else { self.builder.build_float_compare(inkwell::FloatPredicate::OLT, l_val.into_float_value(), r_val.into_float_value(), "lttmp") },
                                    _ => unreachable!(),
                                }.unwrap();
                                self.builder.build_int_z_extend(cond, self.context.i64_type(), "booltmp").unwrap().as_basic_value_enum()
                            }
                        };
                        let (ptr, _) = ssa_storage.get(dest).unwrap();
                        self.builder.build_store(*ptr, res).unwrap();
                    }
                    MirInstruction::Call { dest, name, args } => {
                        let (llvm_func, _ret_type) = if let Some(f) = self.module.get_function(name) {
                            (f, f.get_type().get_return_type().unwrap_or(self.context.i64_type().as_basic_type_enum()))
                        } else if name == "broadcasts" || name == "emit" {
                            let i32_type = self.context.i32_type();
                            let str_ptr_type = self.context.i8_type().ptr_type(inkwell::AddressSpace::default());
                            let fn_type = i32_type.fn_type(&[str_ptr_type.into()], false);
                            (self.module.add_function("puts", fn_type, Some(inkwell::module::Linkage::External)), i32_type.as_basic_type_enum())
                        } else {
                            let actual_name = if let Some(idx) = name.find('_') { &name[..idx] } else { name };
                            if let Some(sig) = self.registry.as_ref().and_then(|r| r.get_signature(actual_name)) {
                                let arg_types: Vec<BasicMetadataTypeEnum> = sig.input_types.iter().map(|t| self.onu_type_to_llvm(t).into()).collect();
                                let ret_llvm_type = self.onu_type_to_llvm(&sig.return_type);
                                let fn_type = if sig.return_type == OnuType::Nothing { self.context.void_type().fn_type(&arg_types, false) } else { ret_llvm_type.fn_type(&arg_types, false) };
                                (self.module.add_function(name, fn_type, Some(inkwell::module::Linkage::External)), ret_llvm_type)
                            } else {
                                let i64_type = self.context.i64_type();
                                let mut arg_types = Vec::new();
                                for _ in 0..args.len() { arg_types.push(i64_type.into()); }
                                let fn_type = i64_type.fn_type(&arg_types, false);
                                (self.module.add_function(name, fn_type, Some(inkwell::module::Linkage::External)), i64_type.as_basic_type_enum())
                            }
                        };
                        let mut llvm_args = Vec::new();
                        for arg in args { llvm_args.push(self.operand_to_llvm(arg, &ssa_storage)?.into()); }
                        let call_target = if name == "broadcasts" || name == "emit" { self.module.get_function("puts").unwrap() } else { llvm_func };
                        let call_res = self.builder.build_call(call_target, &llvm_args, "calltmp").unwrap();
                        let res = match call_res.try_as_basic_value() {
                            inkwell::values::ValueKind::Basic(val) => val,
                            inkwell::values::ValueKind::Instruction(_) => self.context.i64_type().const_int(0, false).as_basic_value_enum()
                        };
                        let (ptr, _) = ssa_storage.get(dest).unwrap();
                        self.builder.build_store(*ptr, res).unwrap();
                    }
                    MirInstruction::Tuple { dest, elements } => {
                        let (ptr, typ) = ssa_storage.get(dest).unwrap();
                        let _struct_type = typ.into_struct_type();
                        for (i, e) in elements.iter().enumerate() {
                            let val = self.operand_to_llvm(e, &ssa_storage)?;
                            let field_ptr = self.builder.build_struct_gep(*ptr, i as u32, &format!("f{}", i)).unwrap();
                            self.builder.build_store(field_ptr, val).unwrap();
                        }
                    }
                    MirInstruction::Index { dest, subject, index } => {
                        let (subj_ptr, _subj_type) = match subject {
                            MirOperand::Variable(id) => ssa_storage.get(id).unwrap(),
                            _ => unreachable!(),
                        };
                        let field_ptr = self.builder.build_struct_gep(*subj_ptr, *index as u32, "idx").unwrap();
                        let val = self.builder.build_load(field_ptr, "ldidx").unwrap();
                        let (ptr, _) = ssa_storage.get(dest).unwrap();
                        self.builder.build_store(*ptr, val).unwrap();
                    }
                    MirInstruction::Emit(_op) => {}
                }
            }

            match &mir_block.terminator {
                MirTerminator::Return(op) => {
                    if mir_func.name == "main" || mir_func.name == "run" {
                        self.builder.build_return(Some(&self.context.i32_type().const_int(0, false))).unwrap();
                    } else if mir_func.return_type == OnuType::Nothing {
                        self.builder.build_return(None).unwrap();
                    } else {
                        let val = self.operand_to_llvm(op, &ssa_storage)?;
                        self.builder.build_return(Some(&val)).unwrap();
                    }
                }
                MirTerminator::Branch(target) => {
                    let target_block = llvm_blocks.get(target).unwrap();
                    self.builder.build_unconditional_branch(*target_block).unwrap();
                }
                MirTerminator::CondBranch { condition, then_block, else_block } => {
                    let cond_val_i64 = self.operand_to_llvm(condition, &ssa_storage)?.into_int_value();
                    let cond_val = self.builder.build_int_cast(cond_val_i64, self.context.bool_type(), "brc").unwrap();
                    let then_bb = llvm_blocks.get(then_block).unwrap();
                    let else_bb = llvm_blocks.get(else_block).unwrap();
                    self.builder.build_conditional_branch(cond_val, *then_bb, *else_bb).unwrap();
                }
                MirTerminator::Unreachable => {
                    self.builder.build_unreachable().unwrap();
                }
            }
        }
        if function.verify(true) { Ok(Some(function)) } else { Err(format!("LLVM Function verification failed for {}", mir_func.name)) }
    }

    fn operand_to_llvm(&self, op: &MirOperand, ssa_storage: &HashMap<usize, (PointerValue<'ctx>, BasicTypeEnum<'ctx>)>) -> Result<BasicValueEnum<'ctx>, String> {
        match op {
            MirOperand::Constant(lit) => match lit {
                MirLiteral::I64(n) => Ok(self.context.i64_type().const_int(*n as u64, true).as_basic_value_enum()),
                MirLiteral::F64(n) => Ok(self.context.f64_type().const_float(*n).as_basic_value_enum()),
                MirLiteral::Boolean(b) => Ok(self.context.bool_type().const_int(if *b { 1 } else { 0 }, false).as_basic_value_enum()),
                MirLiteral::Text(s) => {
                    let global_str = self.builder.build_global_string_ptr(s, "strtmp").unwrap();
                    Ok(global_str.as_basic_value_enum())
                }
                MirLiteral::Nothing => Ok(self.context.i64_type().const_int(0, false).as_basic_value_enum()),
            },
            MirOperand::Variable(id) => {
                let (ptr, _typ) = ssa_storage.get(id).ok_or_else(|| format!("SSA variable {} not found", id))?;
                Ok(self.builder.build_load(*ptr, &format!("ld{}", id)).unwrap())
            },
        }
    }
}

impl<'ctx> CodeGenerator for LlvmGenerator<'ctx> {
    fn generate(&self, program: &MirProgram) -> Result<Vec<u8>, String> {
        for mir_func in &program.functions {
            if mir_func.args.iter().any(|arg| matches!(arg.typ, OnuType::Shape(_) | OnuType::Nothing)) { continue; }
            let arg_types: Vec<BasicMetadataTypeEnum> = mir_func.args.iter().map(|arg| self.onu_type_to_llvm(&arg.typ).into()).collect();
            let fn_name = if mir_func.name == "main" || mir_func.name == "run" { "main" } else { &mir_func.name };
            let fn_type = if fn_name == "main" { self.context.i32_type().fn_type(&arg_types, false) } 
                          else if mir_func.return_type == OnuType::Nothing { self.context.void_type().fn_type(&arg_types, false) } 
                          else { self.onu_type_to_llvm(&mir_func.return_type).fn_type(&arg_types, false) };
            let function = self.module.add_function(fn_name, fn_type, None);
            if fn_name == "main" { function.set_linkage(inkwell::module::Linkage::External); }
        }
        for func in &program.functions { self.generate_function(func)?; }
        self.run_optimizations();
        Ok(self.module.write_bitcode_to_memory().as_slice().to_vec())
    }
}
