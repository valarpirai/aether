use crate::parser::ast::{ArrayDestructureElem, DestructurePattern, Expr, Pattern, Program, Stmt};
use std::collections::HashSet;

pub struct Diagnostic {
    pub line: usize,
    pub message: String,
}

struct Checker {
    scopes: Vec<HashSet<String>>,
    pub diagnostics: Vec<Diagnostic>,
    current_line: usize,
}

impl Checker {
    fn new() -> Self {
        let mut global: HashSet<String> = HashSet::new();
        for name in BUILTINS {
            global.insert((*name).to_string());
        }
        Self {
            scopes: vec![global],
            diagnostics: Vec::new(),
            current_line: 0,
        }
    }

    fn define(&mut self, name: &str) {
        if let Some(top) = self.scopes.last_mut() {
            top.insert(name.to_string());
        }
    }

    fn push_scope(&mut self) {
        self.scopes.push(HashSet::new());
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    fn is_defined(&self, name: &str) -> bool {
        self.scopes.iter().rev().any(|s| s.contains(name))
    }

    fn report(&mut self, msg: String) {
        self.diagnostics.push(Diagnostic {
            line: self.current_line,
            message: msg,
        });
    }

    fn check_stmts(&mut self, stmts: &[Stmt]) {
        for stmt in stmts {
            self.check_stmt(stmt);
        }
    }

    fn check_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Line(n) => self.current_line = *n,
            Stmt::Debugger | Stmt::Break(_) | Stmt::Continue(_) => {}

            Stmt::Let(name, expr) => {
                self.check_expr(expr);
                self.define(name);
            }
            Stmt::LetDestructure {
                pattern,
                initializer,
            } => {
                self.check_expr(initializer);
                self.bind_destructure(pattern);
            }
            Stmt::Assign(target, value) => {
                self.check_expr(target);
                self.check_expr(value);
            }
            Stmt::CompoundAssign(target, _, value) => {
                self.check_expr(target);
                self.check_expr(value);
            }
            Stmt::Block(stmts) => {
                self.push_scope();
                self.check_stmts(stmts);
                self.pop_scope();
            }
            Stmt::If(cond, then_b, else_b) => {
                self.check_expr(cond);
                self.push_scope();
                self.check_stmt(then_b);
                self.pop_scope();
                if let Some(eb) = else_b {
                    self.push_scope();
                    self.check_stmt(eb);
                    self.pop_scope();
                }
            }
            Stmt::While(cond, body) => {
                self.check_expr(cond);
                self.push_scope();
                self.check_stmt(body);
                self.pop_scope();
            }
            Stmt::For(var, iter, body) => {
                self.check_expr(iter);
                self.push_scope();
                self.define(var);
                self.check_stmt(body);
                self.pop_scope();
            }
            Stmt::Labeled(_, inner) => self.check_stmt(inner),
            Stmt::Return(expr) => {
                if let Some(e) = expr {
                    self.check_expr(e);
                }
            }
            Stmt::Throw(expr) => self.check_expr(expr),
            Stmt::Expr(expr) => self.check_expr(expr),

            Stmt::Function(name, params, body) => {
                self.define(name);
                self.push_scope();
                for p in params {
                    self.define(p);
                }
                self.check_stmt(body);
                self.pop_scope();
            }
            Stmt::AsyncFunction(name, params, body) => {
                self.define(name);
                self.push_scope();
                for p in params {
                    self.define(p);
                }
                self.check_stmt(body);
                self.pop_scope();
            }

            Stmt::TryCatch(try_b, err_var, catch_b, finally_b) => {
                self.push_scope();
                self.check_stmt(try_b);
                self.pop_scope();
                self.push_scope();
                self.define(err_var);
                self.check_stmt(catch_b);
                self.pop_scope();
                if let Some(fin) = finally_b {
                    self.push_scope();
                    self.check_stmt(fin);
                    self.pop_scope();
                }
            }

            Stmt::Import(name) => self.define(name),
            Stmt::ImportAs(_, alias) => self.define(alias),
            Stmt::FromImport(_, items) => {
                for item in items {
                    self.define(item);
                }
            }
            Stmt::FromImportAs(_, items) => {
                for (_, alias) in items {
                    self.define(alias);
                }
            }

            Stmt::StructDecl {
                name,
                fields: _,
                methods,
            } => {
                self.define(name);
                for (_, params, body) in methods {
                    self.push_scope();
                    self.define("self");
                    for p in params {
                        self.define(p);
                    }
                    self.check_stmt(body);
                    self.pop_scope();
                }
            }
            Stmt::EnumDecl { name, variants } => {
                self.define(name);
                // Variant constructors are accessed as name::Variant — member access, not identifiers
                let _ = variants;
            }

            Stmt::Match { subject, arms } => {
                self.check_expr(subject);
                for (pat, body) in arms {
                    self.push_scope();
                    self.bind_pattern(pat);
                    self.check_stmt(body);
                    self.pop_scope();
                }
            }
        }
    }

    fn bind_destructure(&mut self, pattern: &DestructurePattern) {
        match pattern {
            DestructurePattern::Array(elems) => {
                for elem in elems {
                    match elem {
                        ArrayDestructureElem::Binding { name, default } => {
                            if let Some(d) = default {
                                self.check_expr(d);
                            }
                            self.define(name);
                        }
                        ArrayDestructureElem::Rest(name) => self.define(name),
                    }
                }
            }
            DestructurePattern::Dict(elems) => {
                for elem in elems {
                    if let Some(d) = &elem.default {
                        self.check_expr(d);
                    }
                    let binding = elem.alias.as_deref().unwrap_or(&elem.key);
                    self.define(binding);
                }
            }
        }
    }

    fn bind_pattern(&mut self, pat: &Pattern) {
        match pat {
            Pattern::Bind(name) => self.define(name),
            Pattern::Or(pats) => {
                for p in pats {
                    self.bind_pattern(p);
                }
            }
            Pattern::EnumVariant(_, _, fields) => {
                for f in fields {
                    self.define(f);
                }
            }
            Pattern::Literal(_) | Pattern::Wildcard => {}
        }
    }

    fn check_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Integer(_) | Expr::Float(_) | Expr::String(_) | Expr::Bool(_) | Expr::Null => {}

            Expr::Identifier(name) => {
                if !self.is_defined(name) {
                    self.report(format!("undefined variable '{}'", name));
                }
            }
            Expr::Binary(l, _, r) => {
                self.check_expr(l);
                self.check_expr(r);
            }
            Expr::Unary(_, operand) => self.check_expr(operand),
            Expr::Call(func, args) => {
                self.check_expr(func);
                for a in args {
                    self.check_expr(a);
                }
            }
            Expr::Array(elems) => {
                for e in elems {
                    self.check_expr(e);
                }
            }
            Expr::Dict(pairs) => {
                for (k, v) in pairs {
                    self.check_expr(k);
                    self.check_expr(v);
                }
            }
            Expr::Index(obj, idx) => {
                self.check_expr(obj);
                self.check_expr(idx);
            }
            Expr::Slice(obj, start, end) => {
                self.check_expr(obj);
                if let Some(s) = start {
                    self.check_expr(s);
                }
                if let Some(e) = end {
                    self.check_expr(e);
                }
            }
            Expr::Spread(inner) => self.check_expr(inner),
            Expr::Member(obj, _) => self.check_expr(obj),
            Expr::OptionalMember(obj, _) => self.check_expr(obj),
            Expr::OptionalCall(obj, _, args) => {
                self.check_expr(obj);
                for a in args {
                    self.check_expr(a);
                }
            }
            Expr::FunctionExpr(params, body) => {
                self.push_scope();
                for p in params {
                    self.define(p);
                }
                self.check_stmt(body);
                self.pop_scope();
            }
            Expr::AsyncFunctionExpr(params, body) => {
                self.push_scope();
                for p in params {
                    self.define(p);
                }
                self.check_stmt(body);
                self.pop_scope();
            }
            Expr::Await(inner) => self.check_expr(inner),
            Expr::StringInterp(parts) => {
                for p in parts {
                    self.check_expr(p);
                }
            }
            Expr::StructInit { name: _, fields } => {
                for (_, v) in fields {
                    self.check_expr(v);
                }
            }
            Expr::Ternary(cond, then_e, else_e) => {
                self.check_expr(cond);
                self.check_expr(then_e);
                self.check_expr(else_e);
            }
        }
    }
}

/// Hoist top-level function, struct, enum, and let names so forward references
/// inside function bodies don't produce false positives.
fn hoist_top_level(stmts: &[Stmt], scope: &mut HashSet<String>) {
    for stmt in stmts {
        match stmt {
            Stmt::Function(name, _, _) | Stmt::AsyncFunction(name, _, _) => {
                scope.insert(name.clone());
            }
            Stmt::StructDecl { name, .. } | Stmt::EnumDecl { name, .. } => {
                scope.insert(name.clone());
            }
            Stmt::Let(name, _) => {
                scope.insert(name.clone());
            }
            Stmt::LetDestructure { .. } => {}
            Stmt::Import(name) => {
                scope.insert(name.clone());
            }
            Stmt::ImportAs(_, alias) => {
                scope.insert(alias.clone());
            }
            Stmt::FromImport(_, items) => {
                for item in items {
                    scope.insert(item.clone());
                }
            }
            Stmt::FromImportAs(_, items) => {
                for (_, alias) in items {
                    scope.insert(alias.clone());
                }
            }
            _ => {}
        }
    }
}

const BUILTINS: &[&str] = &[
    // Native builtins
    "print",
    "println",
    "type",
    "len",
    "int",
    "float",
    "str",
    "bool",
    "hex",
    "oct",
    "bin",
    "base64_encode",
    "base64_decode",
    "read_file",
    "write_file",
    "read_lines",
    "append_file",
    "file_exists",
    "is_file",
    "is_dir",
    "mkdir",
    "lines_iter",
    "read_bytes",
    "write_bytes",
    "list_dir",
    "path_join",
    "rename",
    "rm",
    "input",
    "clock",
    "sleep",
    "set",
    "make_weak",
    "upgrade_weak",
    "is_weak",
    "id",
    "copy",
    "json_parse",
    "json_stringify",
    "http_get",
    "http_post",
    "tcp_listen",
    "tcp_connect",
    "udp_bind",
    "set_workers",
    "args",
    "Promise",
    // Stdlib (auto-loaded at runtime from stdlib/*.ae)
    "range",
    "enumerate",
    "map",
    "filter",
    "reduce",
    "find",
    "every",
    "some",
    "first",
    "last",
    "chunk",
    "partition",
    "zip",
    "zip_longest",
    "flat_map",
    "flatten",
    "take",
    "drop",
    "group_by",
    "count_by",
    "sort",
    "concat",
    "uniq",
    "uniq_by",
    "abs",
    "min",
    "max",
    "sum",
    "sum_by",
    "clamp",
    "sign",
    "floor",
    "ceil",
    "round",
    "trunc",
    "sqrt",
    "pow",
    "log",
    "exp",
    "sin",
    "cos",
    "tan",
    "degrees",
    "radians",
    "hypot",
    "factorial",
    "gcd",
    "lcm",
    "join",
    "repeat",
    "reverse",
    "starts_with",
    "ends_with",
    "contains",
    "index_of",
    "replace",
    "count",
    "pad_left",
    "pad_right",
    "strip_prefix",
    "strip_suffix",
    "is_alpha",
    "is_digit",
    "is_space",
    "test",
    "test_summary",
    "assert_eq",
    "assert_true",
    "assert_false",
    "assert_null",
    "assert_not_null",
    "expect_error",
    // Math constants (defined in math.ae)
    "pi",
    "e",
    "tau",
];

pub fn check(program: &Program) -> Vec<Diagnostic> {
    let mut checker = Checker::new();
    // Hoist top-level names into the global scope so forward references work
    hoist_top_level(&program.statements, checker.scopes.last_mut().unwrap());
    checker.check_stmts(&program.statements);
    checker.diagnostics
}
