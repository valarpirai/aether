use crate::interpreter::environment::RuntimeError;
use crate::interpreter::value::{IteratorSource, Value};
use crate::parser::ast::{Pattern, Stmt};
use std::collections::HashMap;
use std::rc::Rc;

use super::{ControlFlow, Evaluator};

impl Evaluator {
    pub(super) fn exec_stmt_internal(&mut self, stmt: &Stmt) -> Result<ControlFlow, RuntimeError> {
        match stmt {
            Stmt::Expr(expr) => {
                self.eval_expr(expr)?;
                Ok(ControlFlow::None)
            }
            Stmt::Let(name, initializer) => {
                let value = self.eval_expr(initializer)?;
                self.environment.define(name.clone(), value);
                Ok(ControlFlow::None)
            }
            Stmt::Assign(target, value) => {
                let val = self.eval_expr(value)?;
                self.assign_target(target, val)?;
                Ok(ControlFlow::None)
            }
            Stmt::CompoundAssign(target, op, value) => {
                let current = self.eval_expr(target)?;
                let rhs = self.eval_expr(value)?;
                let result = self.eval_binary_values(current, *op, rhs)?;
                self.assign_target(target, result)?;
                Ok(ControlFlow::None)
            }
            Stmt::Block(statements) => {
                for statement in statements {
                    let flow = self.exec_stmt_internal(statement)?;
                    if flow != ControlFlow::None {
                        return Ok(flow);
                    }
                }
                Ok(ControlFlow::None)
            }
            Stmt::If(condition, then_branch, else_branch) => {
                let cond_val = self.eval_expr(condition)?;
                if cond_val.is_truthy() {
                    self.exec_stmt_internal(then_branch)
                } else if let Some(else_stmt) = else_branch {
                    self.exec_stmt_internal(else_stmt)
                } else {
                    Ok(ControlFlow::None)
                }
            }
            Stmt::While(condition, body) => self.exec_while(None, condition, body),
            Stmt::For(var, iterable, body) => self.exec_for(None, var, iterable, body),
            Stmt::Labeled(label, inner) => match inner.as_ref() {
                Stmt::While(condition, body) => self.exec_while(Some(label), condition, body),
                Stmt::For(var, iterable, body) => self.exec_for(Some(label), var, iterable, body),
                other => self.exec_stmt_internal(other),
            },
            Stmt::Return(expr) => {
                let value = if let Some(e) = expr {
                    self.eval_expr(e)?
                } else {
                    Value::Null
                };
                Ok(ControlFlow::Return(value))
            }
            Stmt::Break(label) => Ok(ControlFlow::Break(label.clone())),
            Stmt::Continue(label) => Ok(ControlFlow::Continue(label.clone())),
            Stmt::Function(name, params, body) => {
                let func = Value::Function {
                    params: params.clone(),
                    body: Rc::clone(body),
                    closure: Rc::new(self.environment.clone()),
                };
                self.environment.define(name.clone(), func);
                Ok(ControlFlow::None)
            }
            Stmt::AsyncFunction(name, params, body) => {
                let func = Value::AsyncFunction {
                    params: params.clone(),
                    body: Rc::clone(body),
                    closure: Rc::new(self.environment.clone()),
                };
                self.environment.define(name.clone(), func);
                Ok(ControlFlow::None)
            }
            Stmt::Import(module_name) => {
                self.load_module(module_name)?;
                Ok(ControlFlow::None)
            }
            Stmt::ImportAs(module_name, alias) => {
                self.load_module_as(module_name, alias)?;
                Ok(ControlFlow::None)
            }
            Stmt::FromImport(module_name, items) => {
                self.import_from(module_name, items)?;
                Ok(ControlFlow::None)
            }
            Stmt::FromImportAs(module_name, aliased_items) => {
                self.import_from_as(module_name, aliased_items)?;
                Ok(ControlFlow::None)
            }
            Stmt::Throw(expr) => {
                let value = self.eval_expr(expr)?;
                let msg = format!("{}", value);
                Err(RuntimeError::Thrown(msg))
            }
            Stmt::StructDecl {
                name,
                fields,
                methods,
            } => {
                let mut method_map = HashMap::new();
                for (method_name, params, body) in methods {
                    method_map.insert(method_name.clone(), (params.clone(), body.clone()));
                }
                let struct_def = Value::StructDef {
                    name: name.clone(),
                    fields: fields.clone(),
                    methods: Rc::new(method_map),
                };
                self.environment.define(name.clone(), struct_def);
                Ok(ControlFlow::None)
            }
            Stmt::EnumDecl { name, variants } => {
                let enum_def = Value::EnumDef {
                    name: name.clone(),
                    variants: Rc::new(variants.clone()),
                };
                self.environment.define(name.clone(), enum_def);
                Ok(ControlFlow::None)
            }
            Stmt::TryCatch(try_body, error_var, catch_body, finally_body) => {
                let saved_stack_len = self.calls.stack.len();
                let result = match self.exec_stmt_internal(try_body) {
                    Ok(flow) => Ok(flow),
                    Err(e) => {
                        let error_val = Value::error_val(
                            e.to_string(),
                            &self.calls.stack,
                            self.calls.current_line,
                        );
                        self.calls.stack.truncate(saved_stack_len);
                        self.environment.define(error_var.clone(), error_val);
                        self.exec_stmt_internal(catch_body)
                    }
                };
                // finally always runs, even if catch returned/broke/errored
                if let Some(finally) = finally_body {
                    self.exec_stmt_internal(finally)?;
                }
                result
            }
            Stmt::Match { subject, arms } => {
                let val = self.eval_expr(subject)?;
                for (pattern, body) in arms {
                    if let Some(bindings) = self.match_pattern(pattern, &val) {
                        for (name, bound) in bindings {
                            self.environment.define(name, bound);
                        }
                        return self.exec_stmt_internal(body);
                    }
                }
                Ok(ControlFlow::None)
            }
            Stmt::Line(n) => {
                self.calls.current_line = *n;
                Ok(ControlFlow::None)
            }
        }
    }

    #[allow(clippy::only_used_in_recursion)]
    fn match_pattern(&self, pattern: &Pattern, val: &Value) -> Option<Vec<(String, Value)>> {
        use crate::parser::ast::Expr as AstExpr;
        match pattern {
            Pattern::Wildcard => Some(vec![]),
            Pattern::Bind(name) => Some(vec![(name.clone(), val.clone())]),
            Pattern::Literal(expr) => {
                let matches = match (expr, val) {
                    (AstExpr::Bool(b), Value::Bool(v)) => b == v,
                    (AstExpr::Null, Value::Null) => true,
                    (AstExpr::Integer(n), Value::Int(v)) => n == v,
                    (AstExpr::Float(f), Value::Float(v)) => f == v,
                    (AstExpr::String(s), Value::String(rc)) => s.as_str() == rc.as_ref().as_str(),
                    _ => false,
                };
                if matches {
                    Some(vec![])
                } else {
                    None
                }
            }
            Pattern::EnumVariant(enum_name, variant_opt, field_names) => {
                if let Value::EnumVariant {
                    enum_name: en,
                    variant_name: vn,
                    fields,
                    ..
                } = val
                {
                    let name_matches = en == enum_name;
                    let variant_matches = variant_opt.as_ref().is_none_or(|v| v == vn);
                    if !(name_matches && variant_matches) {
                        return None;
                    }
                    let mut bindings = Vec::new();
                    for (i, fname) in field_names.iter().enumerate() {
                        if fname == "_" {
                            continue;
                        }
                        let bound = fields.get(i).map(|(_, v)| v.clone()).unwrap_or(Value::Null);
                        bindings.push((fname.clone(), bound));
                    }
                    Some(bindings)
                } else {
                    None
                }
            }
            Pattern::Or(alts) => {
                for alt in alts {
                    if let Some(bindings) = self.match_pattern(alt, val) {
                        return Some(bindings);
                    }
                }
                None
            }
        }
    }

    fn exec_while(
        &mut self,
        label: Option<&String>,
        condition: &crate::parser::ast::Expr,
        body: &Stmt,
    ) -> Result<ControlFlow, RuntimeError> {
        loop {
            let cond_val = self.eval_expr(condition)?;
            if !cond_val.is_truthy() {
                break;
            }
            match self.exec_stmt_internal(body)? {
                ControlFlow::Break(ref lbl) if lbl.as_deref() == label.map(|s| s.as_str()) => break,
                ControlFlow::Break(lbl) => return Ok(ControlFlow::Break(lbl)),
                ControlFlow::Continue(ref lbl) if lbl.as_deref() == label.map(|s| s.as_str()) => {
                    continue
                }
                ControlFlow::Continue(lbl) => return Ok(ControlFlow::Continue(lbl)),
                ControlFlow::Return(val) => return Ok(ControlFlow::Return(val)),
                ControlFlow::None => {}
            }
        }
        Ok(ControlFlow::None)
    }

    fn exec_for(
        &mut self,
        label: Option<&String>,
        var: &str,
        iterable: &crate::parser::ast::Expr,
        body: &Stmt,
    ) -> Result<ControlFlow, RuntimeError> {
        let iter_val = self.eval_expr(iterable)?;

        // FileLines is handled lazily to avoid loading the whole file
        if let Value::FileLines(state) = &iter_val {
            loop {
                let next = state
                    .borrow_mut()
                    .next_line()
                    .map_err(RuntimeError::InvalidOperation)?;
                match next {
                    None => break,
                    Some(line) => {
                        self.environment
                            .define(var.to_string(), Value::string(line));
                        match self.exec_stmt_internal(body)? {
                            ControlFlow::Break(ref lbl)
                                if lbl.as_deref() == label.map(|s| s.as_str()) =>
                            {
                                break
                            }
                            ControlFlow::Break(lbl) => return Ok(ControlFlow::Break(lbl)),
                            ControlFlow::Continue(ref lbl)
                                if lbl.as_deref() == label.map(|s| s.as_str()) =>
                            {
                                continue
                            }
                            ControlFlow::Continue(lbl) => return Ok(ControlFlow::Continue(lbl)),
                            ControlFlow::Return(val) => return Ok(ControlFlow::Return(val)),
                            ControlFlow::None => {}
                        }
                    }
                }
            }
            return Ok(ControlFlow::None);
        }

        let items: Vec<Value> = match iter_val {
            Value::Array(ref elements) => elements.iter().cloned().collect(),
            Value::Dict(ref pairs) => pairs.iter().map(|(k, _)| k.clone()).collect(),
            Value::Set(ref elements) => elements.iter().cloned().collect(),
            Value::String(ref s) => s.chars().map(|c| Value::string(c.to_string())).collect(),
            Value::Iterator(ref state) => {
                let mut result = Vec::new();
                loop {
                    let mut st = state.borrow_mut();
                    let val = match &st.source {
                        IteratorSource::Array(arr) => {
                            if st.index < arr.len() {
                                let v = arr[st.index].clone();
                                st.index += 1;
                                Some(v)
                            } else {
                                None
                            }
                        }
                        IteratorSource::DictKeys(pairs) => {
                            if st.index < pairs.len() {
                                let v = pairs[st.index].0.clone();
                                st.index += 1;
                                Some(v)
                            } else {
                                None
                            }
                        }
                        IteratorSource::Set(items) => {
                            if st.index < items.len() {
                                let v = items[st.index].clone();
                                st.index += 1;
                                Some(v)
                            } else {
                                None
                            }
                        }
                    };
                    drop(st);
                    match val {
                        Some(v) => result.push(v),
                        None => break,
                    }
                }
                result
            }
            _ => {
                return Err(RuntimeError::TypeError {
                    expected: "iterable (array, dict, set, string, or iterator)".to_string(),
                    got: iter_val.type_name().to_string(),
                })
            }
        };

        for element in items {
            self.environment.define(var.to_string(), element);
            match self.exec_stmt_internal(body)? {
                ControlFlow::Break(ref lbl) if lbl.as_deref() == label.map(|s| s.as_str()) => break,
                ControlFlow::Break(lbl) => return Ok(ControlFlow::Break(lbl)),
                ControlFlow::Continue(ref lbl) if lbl.as_deref() == label.map(|s| s.as_str()) => {
                    continue
                }
                ControlFlow::Continue(lbl) => return Ok(ControlFlow::Continue(lbl)),
                ControlFlow::Return(val) => return Ok(ControlFlow::Return(val)),
                ControlFlow::None => {}
            }
        }
        Ok(ControlFlow::None)
    }
}
