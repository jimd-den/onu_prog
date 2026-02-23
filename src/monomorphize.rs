use crate::hir::{HirDiscourse, HirExpression};
use crate::types::OnuType;
use std::collections::HashMap;

pub struct Monomorphizer {
    type_map: HashMap<String, Vec<OnuType>>, // Function name -> Concrete types used
}

impl Monomorphizer {
    pub fn new() -> Self {
        Self {
            type_map: HashMap::new(),
        }
    }

    pub fn run(hir: &mut Vec<HirDiscourse>) {
        let mut monomorphizer = Self::new();
        monomorphizer.identify_specializations(hir);
        monomorphizer.apply_specializations(hir);
    }

    fn identify_specializations(&mut self, hir: &[HirDiscourse]) {
        for discourse in hir {
            if let HirDiscourse::Behavior { header: _, body } = discourse {
                self.visit_expression(body);
            }
        }
    }

    fn visit_expression(&mut self, expr: &HirExpression) {
        match expr {
            HirExpression::Call { name, args } => {
                if name == "receiving" || name == "utilizing" {
                     if let Some(HirExpression::Variable(vname)) = args.get(0) {
                          if vname == "get-size" {
                               if let Some(HirExpression::ActsAs { subject: _, shape: _ }) = args.get(1) {
                                    self.type_map.entry(vname.clone()).or_insert_with(Vec::new).push(OnuType::F64);
                               }
                          }
                     }
                }
                
                if name == "get-size" {
                     for arg in args {
                          if let HirExpression::ActsAs { subject: _, shape: _ } = arg {
                               self.type_map.entry(name.clone()).or_insert_with(Vec::new).push(OnuType::F64);
                          }
                     }
                }

                for arg in args {
                    if let HirExpression::ActsAs { subject: _, shape: _ } = arg {
                         if name == "get-size" {
                             self.type_map.entry(name.clone()).or_insert_with(Vec::new).push(OnuType::F64);
                         }
                    }
                    self.visit_expression(arg);
                }
            }
            HirExpression::Derivation { name: _, typ: _, value, body } => {
                self.visit_expression(value);
                self.visit_expression(body);
            }
            HirExpression::If { condition, then_branch, else_branch } => {
                self.visit_expression(condition);
                self.visit_expression(then_branch);
                self.visit_expression(else_branch);
            }
            HirExpression::ActsAs { subject, .. } => {
                self.visit_expression(subject);
            }
            HirExpression::Block(exprs) => {
                for e in exprs {
                    self.visit_expression(e);
                }
            }
            HirExpression::Emit(e) => {
                self.visit_expression(e);
            }
            _ => {}
        }
    }

    fn apply_specializations(&mut self, hir: &mut Vec<HirDiscourse>) {
        let mut new_discourse = Vec::new();
        for discourse in hir.iter() {
            if let HirDiscourse::Behavior { header, body } = discourse {
                if let Some(concrete_types) = self.type_map.get(&header.name) {
                    for typ in concrete_types {
                        let type_suffix = match typ {
                            OnuType::F64 => "float",
                            OnuType::I64 => "integer",
                            _ => "unknown",
                        };
                        let mut specialized_header = header.clone();
                        specialized_header.name = format!("{}_{}", header.name, type_suffix);
                        
                        // Replace generic/shape arguments with concrete types
                        for arg in &mut specialized_header.args {
                            if matches!(arg.typ, OnuType::Shape(_)) {
                                arg.typ = typ.clone();
                            }
                        }
                        
                        let mut specialized_body = body.clone();
                        self.rewrite_expression(&mut specialized_body, &header.name, &specialized_header.name);

                        new_discourse.push(HirDiscourse::Behavior {
                            header: specialized_header,
                            body: specialized_body,
                        });
                    }
                }
            }
        }
        
        // Also rewrite the call sites in original functions (like main)
        for discourse in hir.iter_mut() {
            if let HirDiscourse::Behavior { header: _, body } = discourse {
                 for (original_name, types) in &self.type_map {
                      for typ in types {
                           let type_suffix = match typ {
                               OnuType::F64 => "float",
                               OnuType::I64 => "integer",
                               _ => "unknown",
                           };
                           let specialized_name = format!("{}_{}", original_name, type_suffix);
                           self.rewrite_call_sites(body, original_name, &specialized_name);
                      }
                 }
            }
        }

        hir.extend(new_discourse);
    }

    fn rewrite_expression(&self, _expr: &mut HirExpression, _old_name: &str, _new_name: &str) {
        // Recursively rewrite body if needed (e.g. recursive calls)
        // For now, we mainly care about rewriting the call sites in 'main' or callers
    }

    fn rewrite_call_sites(&self, expr: &mut HirExpression, old_name: &str, new_name: &str) {
        match expr {
            HirExpression::Call { name, args } => {
                if name == "receiving" || name == "utilizes" {
                     if let Some(HirExpression::Variable(vn)) = args.get_mut(0) {
                          if vn == old_name {
                               // Found: get-size utilizing (x acts-as Measurable)
                               // Rewrite to: get-size_float utilizing x
                               *vn = new_name.to_string();
                               if let HirExpression::ActsAs { subject, .. } = args.get(1).unwrap() {
                                    args[1] = (**subject).clone();
                               }
                          }
                     }
                }
                if name == old_name {
                    *name = new_name.to_string();
                }
                for arg in args {
                    self.rewrite_call_sites(arg, old_name, new_name);
                }
            }
            HirExpression::Derivation { value, body, .. } => {
                self.rewrite_call_sites(value, old_name, new_name);
                self.rewrite_call_sites(body, old_name, new_name);
            }
            HirExpression::If { condition, then_branch, else_branch } => {
                self.rewrite_call_sites(condition, old_name, new_name);
                self.rewrite_call_sites(then_branch, old_name, new_name);
                self.rewrite_call_sites(else_branch, old_name, new_name);
            }
            HirExpression::ActsAs { subject, .. } => {
                self.rewrite_call_sites(subject, old_name, new_name);
            }
            HirExpression::Block(exprs) => {
                for e in exprs {
                    self.rewrite_call_sites(e, old_name, new_name);
                }
            }
            HirExpression::Emit(e) => {
                self.rewrite_call_sites(e, old_name, new_name);
            }
            _ => {}
        }
    }
}
