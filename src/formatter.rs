//! Pretty-printer for Aether source code.
//!
//! Parses source to an AST then emits canonical output:
//! - 4-space indentation
//! - Spaces around binary operators
//! - Blank line between top-level function / struct / enum declarations
//! - Minimal parentheses (precedence-driven)

use crate::parser::ast::{
    ArrayDestructureElem, BinaryOp, DestructurePattern, Expr, Pattern, Stmt, UnaryOp,
};

/// Parse `source` and return canonical formatted text, or an error message.
pub fn format_source(source: &str) -> Result<String, String> {
    use crate::lexer::Scanner;
    use crate::parser::Parser;

    let mut scanner = Scanner::new(source);
    let tokens = scanner.scan_tokens().map_err(|e| e.to_string())?;
    let mut parser = Parser::new(tokens);
    let program = parser.parse().map_err(|e| e.to_string())?;

    let mut fmt = Formatter { indent: 0 };
    Ok(fmt.format_program(&program.statements))
}

// ── Formatter struct ──────────────────────────────────────────────────────────

struct Formatter {
    indent: usize,
}

impl Formatter {
    fn ind(&self) -> String {
        "    ".repeat(self.indent)
    }

    // ── Program ───────────────────────────────────────────────────────────────

    fn format_program(&mut self, stmts: &[Stmt]) -> String {
        let mut parts: Vec<String> = Vec::new();
        let mut prev_was_decl = false;

        for stmt in stmts {
            if matches!(stmt, Stmt::Line(_)) {
                continue;
            }
            let is_decl = matches!(
                stmt,
                Stmt::Function(..)
                    | Stmt::AsyncFunction(..)
                    | Stmt::StructDecl { .. }
                    | Stmt::EnumDecl { .. }
            );
            // Blank line between adjacent declarations or before the first declaration
            if !parts.is_empty() && (is_decl || prev_was_decl) {
                parts.push(String::new());
            }
            let s = self.format_stmt(stmt);
            if !s.is_empty() {
                parts.push(s);
            }
            prev_was_decl = is_decl;
        }

        let mut out = parts.join("\n");
        if !out.ends_with('\n') {
            out.push('\n');
        }
        out
    }

    // ── Block helpers ─────────────────────────────────────────────────────────

    fn format_block(&mut self, stmts: &[Stmt]) -> String {
        let real: Vec<&Stmt> = stmts
            .iter()
            .filter(|s| !matches!(s, Stmt::Line(_)))
            .collect();
        if real.is_empty() {
            return "{}".to_string();
        }
        self.indent += 1;
        let mut lines: Vec<String> = Vec::new();
        for s in &real {
            let formatted = self.format_stmt(s);
            if !formatted.is_empty() {
                lines.push(formatted);
            }
        }
        self.indent -= 1;
        format!("{{\n{}\n{}}}", lines.join("\n"), self.ind())
    }

    /// Format a statement that is expected to be a block body.
    fn format_block_stmt(&mut self, stmt: &Stmt) -> String {
        match stmt {
            Stmt::Block(stmts) => self.format_block(stmts),
            Stmt::Line(_) => "{}".to_string(),
            other => {
                self.indent += 1;
                let s = self.format_stmt(other);
                self.indent -= 1;
                format!("{{\n{}\n{}}}", s, self.ind())
            }
        }
    }

    // ── if / else if / else ───────────────────────────────────────────────────

    fn format_if(
        &mut self,
        cond: &Expr,
        then: &Stmt,
        else_opt: &Option<Box<Stmt>>,
        prefix: &str,
    ) -> String {
        let cond_s = self.format_expr(cond);
        let then_s = self.format_block_stmt(then);
        let else_s = match else_opt {
            None => String::new(),
            Some(e) => match e.as_ref() {
                Stmt::If(c, t, e2) => {
                    let inner = self.format_if(c, t.as_ref(), e2, "");
                    format!(" else {}", inner.trim_start())
                }
                other => format!(" else {}", self.format_block_stmt(other)),
            },
        };
        format!("{}if ({}) {}{}", prefix, cond_s, then_s, else_s)
    }

    // ── Expression formatter ──────────────────────────────────────────────────

    fn format_expr(&mut self, expr: &Expr) -> String {
        self.format_expr_prec(expr, 0)
    }

    fn format_expr_prec(&mut self, expr: &Expr, min_prec: u8) -> String {
        match expr {
            Expr::Integer(n) => n.to_string(),
            Expr::Float(f) => format_float(*f),
            Expr::String(s) => format!("\"{}\"", escape_string(s)),
            Expr::Bool(b) => b.to_string(),
            Expr::Null => "null".to_string(),
            Expr::Identifier(name) => name.clone(),

            Expr::Binary(left, op, right) => {
                let prec = binary_op_prec(*op);
                let l = self.format_expr_prec(left, prec);
                // Power is right-associative: equal prec on right is fine
                let r_min = if *op == BinaryOp::Power {
                    prec
                } else {
                    prec + 1
                };
                let r = self.format_expr_prec(right, r_min);
                let s = format!("{} {} {}", l, binary_op_str(*op), r);
                if prec < min_prec {
                    format!("({})", s)
                } else {
                    s
                }
            }

            Expr::Unary(op, operand) => {
                let (prefix, space) = match op {
                    UnaryOp::Negate => ("-", false),
                    UnaryOp::Not => ("not", true),
                    UnaryOp::BitwiseNot => ("~", false),
                };
                // Parenthesize any binary expression under a unary
                let inner = self.format_expr_prec(operand, 13);
                let s = if space {
                    format!("{} {}", prefix, inner)
                } else {
                    format!("{}{}", prefix, inner)
                };
                if 13 < min_prec {
                    format!("({})", s)
                } else {
                    s
                }
            }

            Expr::Call(callee, args) => {
                let callee_s = self.format_expr_prec(callee, 20);
                let mut arg_strs: Vec<String> = Vec::new();
                for a in args {
                    arg_strs.push(self.format_expr(a));
                }
                format!("{}({})", callee_s, arg_strs.join(", "))
            }

            Expr::Array(elems) => {
                if elems.is_empty() {
                    return "[]".to_string();
                }
                let mut items: Vec<String> = Vec::new();
                for e in elems {
                    items.push(self.format_expr(e));
                }
                format!("[{}]", items.join(", "))
            }

            Expr::Dict(pairs) => {
                if pairs.is_empty() {
                    return "{}".to_string();
                }
                let mut items: Vec<String> = Vec::new();
                for (k, v) in pairs {
                    items.push(format!("{}: {}", self.format_expr(k), self.format_expr(v)));
                }
                format!("{{{}}}", items.join(", "))
            }

            Expr::Index(obj, idx) => {
                let obj_s = self.format_expr_prec(obj, 20);
                let idx_s = self.format_expr(idx);
                format!("{}[{}]", obj_s, idx_s)
            }

            Expr::Slice(obj, start, end) => {
                let obj_s = self.format_expr_prec(obj, 20);
                let start_s = match start {
                    Some(e) => self.format_expr(e),
                    None => String::new(),
                };
                let end_s = match end {
                    Some(e) => self.format_expr(e),
                    None => String::new(),
                };
                format!("{}[{}:{}]", obj_s, start_s, end_s)
            }

            Expr::Spread(e) => format!("...{}", self.format_expr(e)),

            Expr::Member(obj, member) => {
                format!("{}.{}", self.format_expr_prec(obj, 20), member)
            }

            Expr::FunctionExpr(params, body) => {
                let params_s = params.join(", ");
                let body_s = self.format_block_stmt(body.as_ref());
                format!("fn({}) {}", params_s, body_s)
            }

            Expr::AsyncFunctionExpr(params, body) => {
                let params_s = params.join(", ");
                let body_s = self.format_block_stmt(body.as_ref());
                format!("async fn({}) {}", params_s, body_s)
            }

            Expr::StringInterp(parts) => {
                let mut out = String::from("\"");
                for part in parts {
                    match part {
                        Expr::String(s) => out.push_str(&escape_string_interp(s)),
                        other => {
                            out.push_str("${");
                            out.push_str(&self.format_expr(other));
                            out.push('}');
                        }
                    }
                }
                out.push('"');
                out
            }

            Expr::StructInit { name, fields } => {
                if fields.is_empty() {
                    return format!("{} {{}}", name);
                }
                let mut items: Vec<String> = Vec::new();
                for (k, v) in fields {
                    items.push(format!("{}: {}", k, self.format_expr(v)));
                }
                format!("{} {{ {} }}", name, items.join(", "))
            }

            Expr::Await(e) => format!("await {}", self.format_expr(e)),

            Expr::OptionalMember(obj, member) => {
                format!("{}?.{}", self.format_expr_prec(obj, 20), member)
            }

            Expr::OptionalCall(obj, method, args) => {
                let mut arg_strs: Vec<String> = Vec::new();
                for a in args {
                    arg_strs.push(self.format_expr(a));
                }
                format!(
                    "{}?.{}({})",
                    self.format_expr_prec(obj, 20),
                    method,
                    arg_strs.join(", ")
                )
            }

            Expr::Ternary(cond, then, else_) => {
                let cond_s = self.format_expr_prec(cond, 1);
                let then_s = self.format_expr(then);
                let else_s = self.format_expr(else_);
                let s = format!("{} ? {} : {}", cond_s, then_s, else_s);
                if 1 < min_prec {
                    format!("({})", s)
                } else {
                    s
                }
            }
        }
    }

    // ── Statement formatter ───────────────────────────────────────────────────

    fn format_stmt(&mut self, stmt: &Stmt) -> String {
        let ind = self.ind();
        match stmt {
            Stmt::Line(_) => String::new(),
            Stmt::Debugger => format!("{}debugger", ind),

            Stmt::Expr(e) => format!("{}{}", ind, self.format_expr(e)),

            Stmt::Let(name, init) => {
                format!("{}let {} = {}", ind, name, self.format_expr(init))
            }

            Stmt::Assign(target, val) => {
                format!(
                    "{}{} = {}",
                    ind,
                    self.format_expr(target),
                    self.format_expr(val)
                )
            }

            Stmt::CompoundAssign(target, op, val) => {
                format!(
                    "{}{} {} {}",
                    ind,
                    self.format_expr(target),
                    compound_op_str(*op),
                    self.format_expr(val)
                )
            }

            Stmt::Block(stmts) => self.format_block(stmts),

            Stmt::If(cond, then, else_opt) => self.format_if(cond, then.as_ref(), else_opt, &ind),

            Stmt::While(cond, body) => {
                let cond_s = self.format_expr(cond);
                format!(
                    "{}while ({}) {}",
                    ind,
                    cond_s,
                    self.format_block_stmt(body.as_ref())
                )
            }

            Stmt::For(var, iter, body) => {
                let iter_s = self.format_expr(iter);
                format!(
                    "{}for {} in {} {}",
                    ind,
                    var,
                    iter_s,
                    self.format_block_stmt(body.as_ref())
                )
            }

            Stmt::Return(None) => format!("{}return", ind),
            Stmt::Return(Some(e)) => format!("{}return {}", ind, self.format_expr(e)),

            Stmt::Break(None) => format!("{}break", ind),
            Stmt::Break(Some(lbl)) => format!("{}break {}", ind, lbl),

            Stmt::Continue(None) => format!("{}continue", ind),
            Stmt::Continue(Some(lbl)) => format!("{}continue {}", ind, lbl),

            Stmt::Labeled(lbl, body) => {
                let body_s = self.format_stmt(body.as_ref());
                format!("{}{}: {}", ind, lbl, body_s.trim_start())
            }

            Stmt::Function(name, params, body) => {
                format!(
                    "{}fn {}({}) {}",
                    ind,
                    name,
                    params.join(", "),
                    self.format_block_stmt(body.as_ref())
                )
            }

            Stmt::AsyncFunction(name, params, body) => {
                format!(
                    "{}async fn {}({}) {}",
                    ind,
                    name,
                    params.join(", "),
                    self.format_block_stmt(body.as_ref())
                )
            }

            Stmt::Import(m) => format!("{}import {}", ind, m),
            Stmt::ImportAs(m, alias) => format!("{}import {} as {}", ind, m, alias),
            Stmt::FromImport(m, items) => {
                format!("{}from {} import {}", ind, m, items.join(", "))
            }
            Stmt::FromImportAs(m, items) => {
                let s: Vec<String> = items
                    .iter()
                    .map(|(i, a)| format!("{} as {}", i, a))
                    .collect();
                format!("{}from {} import {}", ind, m, s.join(", "))
            }

            Stmt::TryCatch(try_b, err_var, catch_b, finally_b) => {
                let t = format!("{}try {}", ind, self.format_block_stmt(try_b.as_ref()));
                let c = format!(
                    " catch({}) {}",
                    err_var,
                    self.format_block_stmt(catch_b.as_ref())
                );
                let f = match finally_b {
                    Some(fb) => format!(" finally {}", self.format_block_stmt(fb.as_ref())),
                    None => String::new(),
                };
                format!("{}{}{}", t, c, f)
            }

            Stmt::Throw(e) => format!("{}throw {}", ind, self.format_expr(e)),

            Stmt::StructDecl {
                name,
                fields,
                methods,
            } => self.format_struct(name, fields, methods, &ind),

            Stmt::EnumDecl { name, variants } => self.format_enum(name, variants, &ind),

            Stmt::Match { subject, arms } => self.format_match(subject, arms, &ind),

            Stmt::LetDestructure {
                pattern,
                initializer,
            } => {
                format!(
                    "{}let {} = {}",
                    ind,
                    format_destructure(pattern),
                    self.format_expr(initializer)
                )
            }
        }
    }

    // ── Struct / Enum / Match ─────────────────────────────────────────────────

    fn format_struct(
        &mut self,
        name: &str,
        fields: &[String],
        methods: &[(String, Vec<String>, Box<Stmt>)],
        ind: &str,
    ) -> String {
        self.indent += 1;
        let inner = self.ind();

        let field_lines = fields
            .iter()
            .map(|f| format!("{}{}", inner, f))
            .collect::<Vec<_>>()
            .join("\n");

        let mut method_lines: Vec<String> = Vec::new();
        for (mname, params, body) in methods {
            method_lines.push(format!(
                "{}fn {}({}) {}",
                inner,
                mname,
                params.join(", "),
                self.format_block_stmt(body.as_ref())
            ));
        }

        self.indent -= 1;

        match (fields.is_empty(), methods.is_empty()) {
            (true, true) => format!("{}struct {} {{}}", ind, name),
            (false, true) => format!("{}struct {} {{\n{}\n{}}}", ind, name, field_lines, ind),
            (true, false) => {
                format!(
                    "{}struct {} {{\n{}\n{}}}",
                    ind,
                    name,
                    method_lines.join("\n"),
                    ind
                )
            }
            (false, false) => format!(
                "{}struct {} {{\n{}\n\n{}\n{}}}",
                ind,
                name,
                field_lines,
                method_lines.join("\n"),
                ind
            ),
        }
    }

    fn format_enum(&mut self, name: &str, variants: &[(String, Vec<String>)], ind: &str) -> String {
        self.indent += 1;
        let inner = self.ind();
        let variant_lines = variants
            .iter()
            .map(|(vname, vfields)| {
                if vfields.is_empty() {
                    format!("{}{}", inner, vname)
                } else {
                    format!("{}{}({})", inner, vname, vfields.join(", "))
                }
            })
            .collect::<Vec<_>>()
            .join("\n");
        self.indent -= 1;
        format!("{}enum {} {{\n{}\n{}}}", ind, name, variant_lines, ind)
    }

    fn format_match(&mut self, subject: &Expr, arms: &[(Pattern, Box<Stmt>)], ind: &str) -> String {
        let subj_s = self.format_expr(subject);
        self.indent += 1;
        let inner = self.ind();
        let mut arm_lines: Vec<String> = Vec::new();
        for (pat, body) in arms {
            let body_s = self.format_stmt(body.as_ref()).trim_start().to_string();
            arm_lines.push(format!("{}{} => {}", inner, format_pattern(pat), body_s));
        }
        self.indent -= 1;
        format!(
            "{}match {} {{\n{}\n{}}}",
            ind,
            subj_s,
            arm_lines.join("\n"),
            ind
        )
    }
}

// ── Free helper functions ─────────────────────────────────────────────────────

fn binary_op_prec(op: BinaryOp) -> u8 {
    match op {
        BinaryOp::NullCoalesce => 1,
        BinaryOp::Or => 2,
        BinaryOp::And => 3,
        BinaryOp::BitwiseOr => 4,
        BinaryOp::BitwiseXor => 5,
        BinaryOp::BitwiseAnd => 6,
        BinaryOp::Equal | BinaryOp::NotEqual => 7,
        BinaryOp::Less | BinaryOp::Greater | BinaryOp::LessEqual | BinaryOp::GreaterEqual => 8,
        BinaryOp::ShiftLeft | BinaryOp::ShiftRight => 9,
        BinaryOp::Add | BinaryOp::Subtract => 10,
        BinaryOp::Multiply | BinaryOp::Divide | BinaryOp::Modulo => 11,
        BinaryOp::Power => 12,
    }
}

fn binary_op_str(op: BinaryOp) -> &'static str {
    match op {
        BinaryOp::Add => "+",
        BinaryOp::Subtract => "-",
        BinaryOp::Multiply => "*",
        BinaryOp::Divide => "/",
        BinaryOp::Modulo => "%",
        BinaryOp::Power => "**",
        BinaryOp::Equal => "==",
        BinaryOp::NotEqual => "!=",
        BinaryOp::Less => "<",
        BinaryOp::Greater => ">",
        BinaryOp::LessEqual => "<=",
        BinaryOp::GreaterEqual => ">=",
        BinaryOp::And => "and",
        BinaryOp::Or => "or",
        BinaryOp::NullCoalesce => "??",
        BinaryOp::BitwiseAnd => "&",
        BinaryOp::BitwiseOr => "|",
        BinaryOp::BitwiseXor => "^",
        BinaryOp::ShiftLeft => "<<",
        BinaryOp::ShiftRight => ">>",
    }
}

fn compound_op_str(op: BinaryOp) -> &'static str {
    match op {
        BinaryOp::Add => "+=",
        BinaryOp::Subtract => "-=",
        BinaryOp::Multiply => "*=",
        BinaryOp::Divide => "/=",
        BinaryOp::Modulo => "%=",
        BinaryOp::Power => "**=",
        BinaryOp::BitwiseAnd => "&=",
        BinaryOp::BitwiseOr => "|=",
        BinaryOp::BitwiseXor => "^=",
        BinaryOp::ShiftLeft => "<<=",
        BinaryOp::ShiftRight => ">>=",
        _ => "+=",
    }
}

fn format_pattern(pat: &Pattern) -> String {
    match pat {
        Pattern::Literal(e) => format_literal_expr(e),
        Pattern::Wildcard => "_".to_string(),
        Pattern::Bind(name) => name.clone(),
        Pattern::EnumVariant(enum_name, variant_opt, fields) => {
            let full = match variant_opt {
                Some(v) => format!("{}.{}", enum_name, v),
                None => enum_name.clone(),
            };
            if fields.is_empty() {
                full
            } else {
                format!("{}({})", full, fields.join(", "))
            }
        }
        Pattern::Or(pats) => pats
            .iter()
            .map(format_pattern)
            .collect::<Vec<_>>()
            .join(" | "),
    }
}

/// Format a literal expression (used in patterns and destructure defaults).
/// No mutable state needed — only handles scalar literals.
fn format_literal_expr(expr: &Expr) -> String {
    match expr {
        Expr::Integer(n) => n.to_string(),
        Expr::Float(f) => format_float(*f),
        Expr::String(s) => format!("\"{}\"", escape_string(s)),
        Expr::Bool(b) => b.to_string(),
        Expr::Null => "null".to_string(),
        Expr::Unary(UnaryOp::Negate, inner) => format!("-{}", format_literal_expr(inner)),
        _ => "?".to_string(),
    }
}

fn format_destructure(pat: &DestructurePattern) -> String {
    match pat {
        DestructurePattern::Array(elems) => {
            let parts: Vec<String> = elems
                .iter()
                .map(|e| match e {
                    ArrayDestructureElem::Binding {
                        name,
                        default: None,
                    } => name.clone(),
                    ArrayDestructureElem::Binding {
                        name,
                        default: Some(d),
                    } => {
                        format!("{} = {}", name, format_literal_expr(d))
                    }
                    ArrayDestructureElem::Rest(name) => format!("...{}", name),
                })
                .collect();
            format!("[{}]", parts.join(", "))
        }
        DestructurePattern::Dict(elems) => {
            let parts: Vec<String> = elems
                .iter()
                .map(|e| {
                    let mut s = e.key.clone();
                    if let Some(alias) = &e.alias {
                        s.push_str(&format!(": {}", alias));
                    }
                    if let Some(def) = &e.default {
                        s.push_str(&format!(" = {}", format_literal_expr(def)));
                    }
                    s
                })
                .collect();
            format!("{{{}}}", parts.join(", "))
        }
    }
}

fn format_float(f: f64) -> String {
    let s = f.to_string();
    if s.contains('.') || s.contains('e') || s.contains('E') {
        s
    } else {
        format!("{}.0", s)
    }
}

fn escape_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

/// Escape a string interpolation segment (same rules as escape_string).
fn escape_string_interp(s: &str) -> String {
    escape_string(s)
}

/// Run `aether fmt [--check] <file>`. Returns exit code (0 = ok, 1 = error / unformatted).
pub fn run_fmt(args: &[String]) -> i32 {
    let mut check = false;
    let mut file: Option<&str> = None;

    for arg in args {
        match arg.as_str() {
            "--check" => check = true,
            f if !f.starts_with('-') => file = Some(f),
            other => {
                eprintln!("fmt: unknown option '{}'", other);
                return 1;
            }
        }
    }

    let path = match file {
        Some(p) => p,
        None => {
            eprintln!("fmt: no file specified");
            eprintln!("Usage: aether fmt [--check] <file>");
            return 1;
        }
    };

    let source = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("fmt: cannot read '{}': {}", path, e);
            return 1;
        }
    };

    let formatted = match format_source(&source) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("fmt: {}: {}", path, e);
            return 1;
        }
    };

    if check {
        if formatted == source {
            0
        } else {
            eprintln!("fmt: '{}' is not formatted", path);
            1
        }
    } else {
        if let Err(e) = std::fs::write(path, &formatted) {
            eprintln!("fmt: cannot write '{}': {}", path, e);
            return 1;
        }
        0
    }
}
