use crate::types::OnuType;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct MirProgram {
    pub functions: Vec<MirFunction>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MirFunction {
    pub name: String,
    pub args: Vec<MirArgument>,
    pub return_type: OnuType,
    pub blocks: Vec<BasicBlock>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MirArgument {
    pub name: String,
    pub typ: OnuType,
    pub ssa_var: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BasicBlock {
    pub id: usize,
    pub instructions: Vec<MirInstruction>,
    pub terminator: MirTerminator,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MirInstruction {
    Assign { dest: usize, src: MirOperand },
    BinaryOperation { dest: usize, op: MirBinOp, lhs: MirOperand, rhs: MirOperand },
    Call { dest: usize, name: String, args: Vec<MirOperand> },
    Tuple { dest: usize, elements: Vec<MirOperand> },
    Index { dest: usize, subject: MirOperand, index: usize },
    Emit(MirOperand),
}

#[derive(Debug, Clone, PartialEq)]
pub enum MirBinOp {
    Add, Sub, Mul, Div, Eq, Gt, Lt,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MirOperand {
    Constant(MirLiteral),
    Variable(usize),
}

#[derive(Debug, Clone, PartialEq)]
pub enum MirLiteral {
    I64(i64),
    F64(f64),
    Boolean(bool),
    Text(String),
    Nothing,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MirTerminator {
    Return(MirOperand),
    Branch(usize), // block id
    CondBranch { condition: MirOperand, then_block: usize, else_block: usize },
    Unreachable,
}

pub struct MirBuilder {
    next_ssa_var: usize,
    next_block_id: usize,
    var_map: HashMap<String, usize>, // variable name -> ssa var
}

impl MirBuilder {
    pub fn new() -> Self {
        Self {
            next_ssa_var: 0,
            next_block_id: 0,
            var_map: HashMap::new(),
        }
    }

    fn new_ssa_var(&mut self) -> usize {
        let var = self.next_ssa_var;
        self.next_ssa_var += 1;
        var
    }

    fn new_block_id(&mut self) -> usize {
        let id = self.next_block_id;
        self.next_block_id += 1;
        id
    }

    pub fn build_program(hir: &[crate::hir::HirDiscourse]) -> MirProgram {
        let mut builder = Self::new();
        let mut functions = Vec::new();
        for discourse in hir {
            if let crate::hir::HirDiscourse::Behavior { header, body } = discourse {
                functions.push(builder.build_function(header, body));
            }
        }
        MirProgram { functions }
    }

    pub fn build_program_with_registry(&mut self, hir: &[crate::hir::HirDiscourse], _registry: &crate::registry::Registry) -> MirProgram {
        let mut functions = Vec::new();
        for discourse in hir {
            if let crate::hir::HirDiscourse::Behavior { header, body } = discourse {
                functions.push(self.build_function(header, body));
            }
        }
        MirProgram { functions }
    }

    fn build_function(&mut self, header: &crate::hir::HirBehaviorHeader, body: &crate::hir::HirExpression) -> MirFunction {
        self.var_map.clear();
        self.next_ssa_var = 0;
        self.next_block_id = 0;
        let mut args = Vec::new();
        for arg in &header.args {
            let ssa_var = self.new_ssa_var();
            self.var_map.insert(arg.name.clone(), ssa_var);
            args.push(MirArgument {
                name: arg.name.clone(),
                typ: arg.typ.clone(),
                ssa_var,
            });
        }

        let mut blocks = Vec::new();
        let mut current_block = BasicBlock {
            id: self.new_block_id(),
            instructions: Vec::new(),
            terminator: MirTerminator::Unreachable,
        };

        let result_op = self.build_expression(body, &mut current_block, &mut blocks);
        current_block.terminator = MirTerminator::Return(result_op);
        blocks.push(current_block);

        MirFunction {
            name: header.name.clone(),
            args,
            return_type: header.return_type.clone(),
            blocks,
        }
    }

    fn build_expression(&mut self, expr: &crate::hir::HirExpression, current_block: &mut BasicBlock, blocks: &mut Vec<BasicBlock>) -> MirOperand {
        match expr {
            crate::hir::HirExpression::Literal(lit) => MirOperand::Constant(match lit {
                crate::hir::HirLiteral::I64(n) => MirLiteral::I64(*n),
                crate::hir::HirLiteral::F64(n) => MirLiteral::F64(*n),
                crate::hir::HirLiteral::Boolean(b) => MirLiteral::Boolean(*b),
                crate::hir::HirLiteral::Text(s) => MirLiteral::Text(s.clone()),
                crate::hir::HirLiteral::Nothing => MirLiteral::Nothing,
            }),
            crate::hir::HirExpression::Variable(name) => {
                let ssa_var = *self.var_map.get(name).unwrap_or_else(|| {
                    panic!("Variable '{}' not found in MIR build. Var map: {:?}", name, self.var_map);
                });
                MirOperand::Variable(ssa_var)
            }
            crate::hir::HirExpression::Call { name, args } => {
                let mut mir_args = Vec::new();
                for arg in args {
                    mir_args.push(self.build_expression(arg, current_block, blocks));
                }
                
                let bin_op = if mir_args.len() == 2 {
                    match name.as_str() {
                        "added-to" => Some(MirBinOp::Add),
                        "decreased-by" => Some(MirBinOp::Sub),
                        "scales-by" => Some(MirBinOp::Mul),
                        "partitions-by" => Some(MirBinOp::Div),
                        "matches" => Some(MirBinOp::Eq),
                        "exceeds" => Some(MirBinOp::Gt),
                        "falls-short-of" => Some(MirBinOp::Lt),
                        _ => None,
                    }
                } else {
                    None
                };

                let dest = self.new_ssa_var();
                if let Some(op) = bin_op {
                    current_block.instructions.push(MirInstruction::BinaryOperation {
                        dest,
                        op,
                        lhs: mir_args[0].clone(),
                        rhs: mir_args[1].clone(),
                    });
                } else {
                    current_block.instructions.push(MirInstruction::Call { dest, name: name.clone(), args: mir_args });
                }
                MirOperand::Variable(dest)
            }
            crate::hir::HirExpression::Derivation { name, value, body, .. } => {
                let val_op = self.build_expression(value, current_block, blocks);
                let dest = self.new_ssa_var();
                current_block.instructions.push(MirInstruction::Assign { dest, src: val_op });
                self.var_map.insert(name.clone(), dest);
                self.build_expression(body, current_block, blocks)
            }
            crate::hir::HirExpression::If { condition, then_branch, else_branch } => {
                let cond_op = self.build_expression(condition, current_block, blocks);
                let dest = self.new_ssa_var();
                
                let then_id = self.new_block_id();
                let else_id = self.new_block_id();
                let merge_id = self.new_block_id();
                
                current_block.terminator = MirTerminator::CondBranch { condition: cond_op, then_block: then_id, else_block: else_id };
                
                // Finalize the current block by pushing it
                let mut old_current = std::mem::replace(current_block, BasicBlock { id: then_id, instructions: Vec::new(), terminator: MirTerminator::Unreachable });
                blocks.push(old_current);
                
                // Then Branch
                let then_res = self.build_expression(then_branch, current_block, blocks);
                current_block.terminator = MirTerminator::Branch(merge_id);
                let mut then_finalized = std::mem::replace(current_block, BasicBlock { id: else_id, instructions: Vec::new(), terminator: MirTerminator::Unreachable });
                then_finalized.instructions.push(MirInstruction::Assign { dest, src: then_res });
                blocks.push(then_finalized);
                
                // Else Branch
                let else_res = self.build_expression(else_branch, current_block, blocks);
                current_block.terminator = MirTerminator::Branch(merge_id);
                let mut else_finalized = std::mem::replace(current_block, BasicBlock { id: merge_id, instructions: Vec::new(), terminator: MirTerminator::Unreachable });
                else_finalized.instructions.push(MirInstruction::Assign { dest, src: else_res });
                blocks.push(else_finalized);
                
                // The 'current_block' is now the merge block (merge_id)
                MirOperand::Variable(dest)
            }
            crate::hir::HirExpression::Block(exprs) => {
                let mut last_res = MirOperand::Constant(MirLiteral::Nothing);
                for e in exprs { last_res = self.build_expression(e, current_block, blocks); }
                last_res
            }
            crate::hir::HirExpression::Emit(e) => {
                let op = self.build_expression(e, current_block, blocks);
                current_block.instructions.push(MirInstruction::Emit(op));
                MirOperand::Constant(MirLiteral::Nothing)
            }
            crate::hir::HirExpression::Tuple(elements) => {
                let mut mir_elements = Vec::new();
                for e in elements {
                    mir_elements.push(self.build_expression(e, current_block, blocks));
                }
                let dest = self.new_ssa_var();
                current_block.instructions.push(MirInstruction::Tuple { dest, elements: mir_elements });
                MirOperand::Variable(dest)
            }
            crate::hir::HirExpression::Index { subject, index } => {
                let subj_op = self.build_expression(subject, current_block, blocks);
                let dest = self.new_ssa_var();
                current_block.instructions.push(MirInstruction::Index { dest, subject: subj_op, index: *index });
                MirOperand::Variable(dest)
            }
            crate::hir::HirExpression::ActsAs { subject, .. } => self.build_expression(subject, current_block, blocks),
        }
    }
}
