use crate::interpreter::environment::{Environment, RuntimeError};
use crate::interpreter::value::Value;
use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;

use super::Evaluator;

impl Evaluator {
    pub(super) fn load_module(&mut self, module_name: &str) -> Result<(), RuntimeError> {
        let module_val = self.load_module_as_value(module_name)?;
        self.environment.define(module_name.to_string(), module_val);
        Ok(())
    }

    pub(super) fn load_module_as(
        &mut self,
        module_name: &str,
        alias: &str,
    ) -> Result<(), RuntimeError> {
        let module_val = self.load_module_as_value(module_name)?;
        self.environment.define(alias.to_string(), module_val);
        Ok(())
    }

    pub(super) fn load_module_as_value(
        &mut self,
        module_name: &str,
    ) -> Result<Value, RuntimeError> {
        if self
            .modules
            .loading_stack
            .contains(&module_name.to_string())
        {
            return Err(RuntimeError::CircularImport {
                module: module_name.to_string(),
            });
        }

        if self.modules.cache.contains_key(module_name) {
            let env = self.modules.cache[module_name].clone();
            return Ok(Self::module_value_from_env(module_name, &env));
        }

        self.modules.loading_stack.push(module_name.to_string());

        let module_path = self.resolve_module_path(module_name)?;
        let module_env = self.execute_module_file(&module_path)?;

        self.modules.loading_stack.retain(|m| m != module_name);

        self.modules
            .cache
            .insert(module_name.to_string(), module_env.clone());

        Ok(Self::module_value_from_env(module_name, &module_env))
    }

    pub(super) fn module_value_from_env(name: &str, env: &Environment) -> Value {
        let members: HashMap<String, Value> = env
            .bindings()
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        Value::Module {
            name: name.to_string(),
            members: Rc::new(members),
        }
    }

    pub(super) fn import_from(
        &mut self,
        module_name: &str,
        items: &[String],
    ) -> Result<(), RuntimeError> {
        if !self.modules.cache.contains_key(module_name) {
            let module_path = self.resolve_module_path(module_name)?;
            let module_env = self.execute_module_file(&module_path)?;
            self.modules
                .cache
                .insert(module_name.to_string(), module_env);
        }

        let module_env = &self.modules.cache[module_name];

        for item in items {
            match module_env.get(item) {
                Ok(value) => {
                    self.environment.define(item.clone(), value);
                }
                Err(_) => {
                    return Err(RuntimeError::PropertyNotFound {
                        type_name: format!("module '{}'", module_name),
                        property: item.clone(),
                    });
                }
            }
        }

        Ok(())
    }

    pub(super) fn import_from_as(
        &mut self,
        module_name: &str,
        items: &[(String, String)],
    ) -> Result<(), RuntimeError> {
        if !self.modules.cache.contains_key(module_name) {
            let module_path = self.resolve_module_path(module_name)?;
            let module_env = self.execute_module_file(&module_path)?;
            self.modules
                .cache
                .insert(module_name.to_string(), module_env);
        }

        let module_env = &self.modules.cache[module_name];

        for (item, alias) in items {
            match module_env.get(item) {
                Ok(value) => {
                    self.environment.define(alias.clone(), value);
                }
                Err(_) => {
                    return Err(RuntimeError::PropertyNotFound {
                        type_name: format!("module '{}'", module_name),
                        property: item.clone(),
                    });
                }
            }
        }

        Ok(())
    }

    pub(super) fn resolve_module_path(&self, module_name: &str) -> Result<PathBuf, RuntimeError> {
        let mut path = PathBuf::from(format!("{}.ae", module_name));
        if path.exists() {
            return Ok(path);
        }

        if let Some(ref current) = self.current_file {
            if let Some(parent) = current.parent() {
                path = parent.join(format!("{}.ae", module_name));
                if path.exists() {
                    return Ok(path);
                }
            }
        }

        path = PathBuf::from(format!("modules/{}.ae", module_name));
        if path.exists() {
            return Ok(path);
        }

        Err(RuntimeError::ModuleNotFound {
            module: module_name.to_string(),
        })
    }

    pub(super) fn execute_module_file(
        &mut self,
        path: &PathBuf,
    ) -> Result<Environment, RuntimeError> {
        use crate::lexer::Scanner;
        use crate::parser::Parser;
        use std::fs;

        let module_name = path.to_string_lossy().to_string();
        let source = fs::read_to_string(path).map_err(|e| RuntimeError::ModuleLoadError {
            module: module_name.clone(),
            reason: e.to_string(),
        })?;

        let mut scanner = Scanner::new(&source);
        let tokens = scanner
            .scan_tokens()
            .map_err(|e| RuntimeError::ModuleLoadError {
                module: module_name.clone(),
                reason: e.to_string(),
            })?;

        let mut parser = Parser::new(tokens);
        let program = parser.parse().map_err(|e| RuntimeError::ModuleLoadError {
            module: module_name.clone(),
            reason: e.to_string(),
        })?;

        let saved_env = self.environment.clone();
        let saved_file = self.current_file.clone();

        self.environment = Environment::new();
        self.current_file = Some(path.clone());

        for stmt in &program.statements {
            if let Err(e) = self.exec_stmt(stmt) {
                self.environment = saved_env;
                self.current_file = saved_file;
                return Err(e);
            }
        }

        let module_env = self.environment.clone();

        self.environment = saved_env;
        self.current_file = saved_file;

        Ok(module_env)
    }
}
