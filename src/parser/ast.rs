//! Abstract Syntax Tree node definitions

use std::rc::Rc;

/// Expression AST node
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// Integer literal
    Integer(i64),
    /// Float literal
    Float(f64),
    /// String literal
    String(String),
    /// Boolean literal
    Bool(bool),
    /// Null literal
    Null,
    /// Variable identifier
    Identifier(String),
    /// Binary operation (left, operator, right)
    Binary(Box<Expr>, BinaryOp, Box<Expr>),
    /// Unary operation (operator, operand)
    Unary(UnaryOp, Box<Expr>),
    /// Function call (function, arguments)
    Call(Box<Expr>, Vec<Expr>),
    /// Array literal
    Array(Vec<Expr>),
    /// Dictionary literal (key-value pairs)
    Dict(Vec<(Expr, Expr)>),
    /// Index access (object, index)
    Index(Box<Expr>, Box<Expr>),
    /// Slice access (object, start, end) — arr[start:end], arr[start:], arr[:end], arr[:]
    Slice(Box<Expr>, Option<Box<Expr>>, Option<Box<Expr>>),
    /// Spread expression: ...expr — valid only inside array literals
    Spread(Box<Expr>),
    /// Member access (object, member)
    Member(Box<Expr>, String),
    /// Function expression (parameters, body)
    FunctionExpr(Vec<String>, Rc<Stmt>),
    /// String interpolation: parts are alternating literals and expressions
    StringInterp(Vec<Expr>),
    /// Struct instantiation: StructName { field: value, ... }
    StructInit {
        name: String,
        fields: Vec<(String, Expr)>,
    },
    /// Async function expression (parameters, body)
    AsyncFunctionExpr(Vec<String>, Rc<Stmt>),
    /// Await expression: await <expr>
    Await(Box<Expr>),
    /// Optional member access: expr?.member — null if expr is null
    OptionalMember(Box<Expr>, String),
    /// Optional method call: expr?.method(args) — null if expr is null
    OptionalCall(Box<Expr>, String, Vec<Expr>),
    /// Ternary expression: condition ? then_expr : else_expr
    Ternary(Box<Expr>, Box<Expr>, Box<Expr>),
}

/// Binary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    // Arithmetic
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,

    // Comparison
    Equal,
    NotEqual,
    Less,
    Greater,
    LessEqual,
    GreaterEqual,

    // Logical
    And,
    Or,
    /// Null coalescing: a ?? b — returns a if not null, else b
    NullCoalesce,

    // Bitwise
    BitwiseAnd,
    BitwiseOr,
    BitwiseXor,
    ShiftLeft,
    ShiftRight,

    /// Exponentiation: a ** b
    Power,
}

/// Unary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Negate,
    Not,
    BitwiseNot,
}

/// Statement AST node
#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    /// Expression statement
    Expr(Expr),
    /// Variable declaration (name, initializer)
    Let(String, Expr),
    /// Assignment (target, value)
    Assign(Expr, Expr),
    /// Compound assignment (target, operator, value)
    CompoundAssign(Expr, BinaryOp, Expr),
    /// Block statement (statements)
    Block(Vec<Stmt>),
    /// If statement (condition, then_branch, else_branch)
    If(Expr, Box<Stmt>, Option<Box<Stmt>>),
    /// While loop (condition, body)
    While(Expr, Box<Stmt>),
    /// For loop (variable, iterable, body)
    For(String, Expr, Box<Stmt>),
    /// Return statement
    Return(Option<Expr>),
    /// Break statement (optional label)
    Break(Option<String>),
    /// Continue statement (optional label)
    Continue(Option<String>),
    /// Labeled loop — wraps a While or For with a label name
    Labeled(String, Box<Stmt>),
    /// Function declaration (name, parameters, body)
    Function(String, Vec<String>, Rc<Stmt>),
    /// Async function declaration (name, parameters, body)
    AsyncFunction(String, Vec<String>, Rc<Stmt>),
    /// Import statement (module_name)
    Import(String),
    /// Import with alias (module_name, alias)
    ImportAs(String, String),
    /// From import (module_name, items)
    FromImport(String, Vec<String>),
    /// From import with aliases (module_name, [(item, alias)])
    FromImportAs(String, Vec<(String, String)>),
    /// Try/catch/finally statement (try_body, error_var, catch_body, finally_body)
    TryCatch(Box<Stmt>, String, Box<Stmt>, Option<Box<Stmt>>),
    /// Throw statement (value to throw)
    Throw(Expr),
    /// Struct declaration (name, fields, methods)
    StructDecl {
        name: String,
        fields: Vec<String>,
        methods: Vec<(String, Vec<String>, Box<Stmt>)>,
    },
    /// Enum declaration (name, variants) — each variant is (name, field_names)
    EnumDecl {
        name: String,
        variants: Vec<(String, Vec<String>)>,
    },
    /// match expr { pattern => body, ... }
    Match {
        subject: Expr,
        arms: Vec<(Pattern, Box<Stmt>)>,
    },
    /// Destructuring let: let [a, b] = arr  /  let {x, y} = dict
    LetDestructure {
        pattern: DestructurePattern,
        initializer: Expr,
    },
    /// Line number marker — injected by the parser, updates evaluator's current_line
    Line(usize),
    /// Debugger breakpoint — pauses execution and opens the interactive debug REPL
    Debugger,
}

/// Top-level pattern for destructuring let statements
#[derive(Debug, Clone, PartialEq)]
pub enum DestructurePattern {
    Array(Vec<ArrayDestructureElem>),
    Dict(Vec<DictDestructureElem>),
}

/// One element inside an array destructure pattern
#[derive(Debug, Clone, PartialEq)]
pub enum ArrayDestructureElem {
    /// Normal binding, with optional default: `a` or `a = 0`
    Binding { name: String, default: Option<Expr> },
    /// Rest binding: `...tail`
    Rest(String),
}

/// One entry inside a dict destructure pattern
#[derive(Debug, Clone, PartialEq)]
pub struct DictDestructureElem {
    /// Key to look up in the dict
    pub key: String,
    /// Renamed binding: `{port: p}` → alias = Some("p")
    pub alias: Option<String>,
    /// Default when key is absent or null: `{x = 0}`
    pub default: Option<Expr>,
}

/// Pattern for match arms
#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    /// Literal value: 1, "hello", true, null
    Literal(Expr),
    /// Catch-all: _
    Wildcard,
    /// Bind to variable: x (non-underscore identifier not followed by {})
    Bind(String),
    /// Enum variant with fields: Some(x), None, Status::Ok
    EnumVariant(String, Option<String>, Vec<String>),
    /// Or-pattern: pat1 | pat2
    Or(Vec<Pattern>),
}

/// Program (top-level statements)
#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub statements: Vec<Stmt>,
}

impl Program {
    pub fn new(statements: Vec<Stmt>) -> Self {
        Self { statements }
    }
}

// ── AST printer ───────────────────────────────────────────────────────────────

/// Run `aether ast [--json] [--output <file>] <file>`. Returns exit code.
pub fn run_ast(args: &[String]) -> i32 {
    use super::parse::Parser;
    use crate::lexer::Scanner;

    let mut json = false;
    let mut output_path: Option<String> = None;
    let mut input_file: Option<&str> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--json" => json = true,
            "--output" => {
                i += 1;
                match args.get(i) {
                    Some(p) => output_path = Some(p.clone()),
                    None => {
                        eprintln!("ast: --output requires a path");
                        return 1;
                    }
                }
            }
            f if !f.starts_with('-') => input_file = Some(f),
            other => {
                eprintln!("ast: unknown option '{}'", other);
                return 1;
            }
        }
        i += 1;
    }

    let path = match input_file {
        Some(p) => p,
        None => {
            eprintln!("ast: no file specified");
            eprintln!("Usage: aether ast [--json] [--output <file>] <file>");
            return 1;
        }
    };

    let source = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("ast: cannot read '{}': {}", path, e);
            return 1;
        }
    };

    let mut scanner = Scanner::new(&source);
    let tokens = match scanner.scan_tokens() {
        Ok(t) => t,
        Err(e) => {
            eprintln!("ast: {}: {}", path, e);
            return 1;
        }
    };
    let mut parser = Parser::new(tokens);
    let program = match parser.parse() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("ast: {}: {}", path, e);
            return 1;
        }
    };

    let output = if json {
        let v = program_to_json(&program);
        serde_json::to_string_pretty(&v).unwrap_or_default()
    } else {
        program_to_tree(&program)
    };

    match output_path {
        Some(ref p) => {
            if let Err(e) = std::fs::write(p, &output) {
                eprintln!("ast: cannot write '{}': {}", p, e);
                return 1;
            }
        }
        None => print!("{}", output),
    }
    0
}

// ── Indented tree format ──────────────────────────────────────────────────────

pub fn program_to_tree(program: &Program) -> String {
    let mut out = String::from("Program\n");
    for stmt in &program.statements {
        if matches!(stmt, Stmt::Line(_)) {
            continue;
        }
        out.push_str(&stmt_to_tree(stmt, 1));
        out.push('\n');
    }
    out
}

fn ind(depth: usize) -> String {
    "  ".repeat(depth)
}

fn stmt_to_tree(stmt: &Stmt, depth: usize) -> String {
    let i = ind(depth);
    match stmt {
        Stmt::Expr(e) => expr_to_tree(e, depth),
        Stmt::Let(name, expr) => {
            format!("{}Let {:?}\n{}", i, name, expr_to_tree(expr, depth + 1))
        }
        Stmt::Assign(target, value) => {
            format!(
                "{}Assign\n{}\n{}",
                i,
                expr_to_tree(target, depth + 1),
                expr_to_tree(value, depth + 1)
            )
        }
        Stmt::CompoundAssign(target, op, value) => {
            format!(
                "{}CompoundAssign {:?}\n{}\n{}",
                i,
                op,
                expr_to_tree(target, depth + 1),
                expr_to_tree(value, depth + 1)
            )
        }
        Stmt::Block(stmts) => {
            let children: Vec<String> = stmts
                .iter()
                .filter(|s| !matches!(s, Stmt::Line(_)))
                .map(|s| stmt_to_tree(s, depth + 1))
                .collect();
            format!("{}Block\n{}", i, children.join("\n"))
        }
        Stmt::If(cond, then, else_opt) => {
            let mut s = format!(
                "{}If\n{}\n{}",
                i,
                expr_to_tree(cond, depth + 1),
                stmt_to_tree(then, depth + 1)
            );
            if let Some(e) = else_opt {
                s.push('\n');
                s.push_str(&stmt_to_tree(e, depth + 1));
            }
            s
        }
        Stmt::While(cond, body) => {
            format!(
                "{}While\n{}\n{}",
                i,
                expr_to_tree(cond, depth + 1),
                stmt_to_tree(body, depth + 1)
            )
        }
        Stmt::For(var, iter, body) => {
            format!(
                "{}For {:?}\n{}\n{}",
                i,
                var,
                expr_to_tree(iter, depth + 1),
                stmt_to_tree(body, depth + 1)
            )
        }
        Stmt::Return(Some(expr)) => {
            format!("{}Return\n{}", i, expr_to_tree(expr, depth + 1))
        }
        Stmt::Return(None) => format!("{}Return", i),
        Stmt::Break(Some(l)) => format!("{}Break {:?}", i, l),
        Stmt::Break(None) => format!("{}Break", i),
        Stmt::Continue(Some(l)) => format!("{}Continue {:?}", i, l),
        Stmt::Continue(None) => format!("{}Continue", i),
        Stmt::Labeled(label, inner) => {
            format!("{}Label {:?}\n{}", i, label, stmt_to_tree(inner, depth + 1))
        }
        Stmt::Function(name, params, body) => {
            format!(
                "{}Function {:?} [{}]\n{}",
                i,
                name,
                params.join(", "),
                stmt_to_tree(body, depth + 1)
            )
        }
        Stmt::AsyncFunction(name, params, body) => {
            format!(
                "{}AsyncFunction {:?} [{}]\n{}",
                i,
                name,
                params.join(", "),
                stmt_to_tree(body, depth + 1)
            )
        }
        Stmt::Import(module) => format!("{}Import {:?}", i, module),
        Stmt::ImportAs(module, alias) => format!("{}ImportAs {:?} as {:?}", i, module, alias),
        Stmt::FromImport(module, items) => {
            format!("{}FromImport {:?} [{}]", i, module, items.join(", "))
        }
        Stmt::FromImportAs(module, items) => {
            let pairs: Vec<String> = items
                .iter()
                .map(|(k, v)| format!("{} as {}", k, v))
                .collect();
            format!("{}FromImportAs {:?} [{}]", i, module, pairs.join(", "))
        }
        Stmt::TryCatch(try_body, err_var, catch_body, finally_opt) => {
            let mut s = format!(
                "{}TryCatch {:?}\n{}\n{}",
                i,
                err_var,
                stmt_to_tree(try_body, depth + 1),
                stmt_to_tree(catch_body, depth + 1)
            );
            if let Some(f) = finally_opt {
                s.push('\n');
                s.push_str(&stmt_to_tree(f, depth + 1));
            }
            s
        }
        Stmt::Throw(expr) => format!("{}Throw\n{}", i, expr_to_tree(expr, depth + 1)),
        Stmt::StructDecl {
            name,
            fields,
            methods,
        } => {
            let mut s = format!("{}Struct {:?} [{}]", i, name, fields.join(", "));
            for (mname, params, body) in methods {
                s.push_str(&format!(
                    "\n{}  Method {:?} [{}]\n{}",
                    i,
                    mname,
                    params.join(", "),
                    stmt_to_tree(body, depth + 2)
                ));
            }
            s
        }
        Stmt::EnumDecl { name, variants } => {
            let mut s = format!("{}Enum {:?}", i, name);
            for (vname, fields) in variants {
                if fields.is_empty() {
                    s.push_str(&format!("\n{}  Variant {:?}", i, vname));
                } else {
                    s.push_str(&format!(
                        "\n{}  Variant {:?} [{}]",
                        i,
                        vname,
                        fields.join(", ")
                    ));
                }
            }
            s
        }
        Stmt::Match { subject, arms } => {
            let mut s = format!("{}Match\n{}", i, expr_to_tree(subject, depth + 1));
            for (pat, body) in arms {
                s.push_str(&format!(
                    "\n{}  Arm {:?}\n{}",
                    i,
                    pat,
                    stmt_to_tree(body, depth + 2)
                ));
            }
            s
        }
        Stmt::LetDestructure {
            pattern,
            initializer,
        } => {
            format!(
                "{}LetDestructure {:?}\n{}",
                i,
                pattern,
                expr_to_tree(initializer, depth + 1)
            )
        }
        Stmt::Line(_) => String::new(),
        Stmt::Debugger => format!("{}Debugger", i),
    }
}

fn expr_to_tree(expr: &Expr, depth: usize) -> String {
    let i = ind(depth);
    match expr {
        Expr::Integer(n) => format!("{}Integer {}", i, n),
        Expr::Float(f) => format!("{}Float {}", i, f),
        Expr::String(s) => format!("{}String {:?}", i, s),
        Expr::Bool(b) => format!("{}Bool {}", i, b),
        Expr::Null => format!("{}Null", i),
        Expr::Identifier(name) => format!("{}Identifier {:?}", i, name),
        Expr::Binary(left, op, right) => {
            format!(
                "{}Binary {:?}\n{}\n{}",
                i,
                op,
                expr_to_tree(left, depth + 1),
                expr_to_tree(right, depth + 1)
            )
        }
        Expr::Unary(op, operand) => {
            format!("{}Unary {:?}\n{}", i, op, expr_to_tree(operand, depth + 1))
        }
        Expr::Call(callee, args) => {
            let mut s = format!("{}Call\n{}", i, expr_to_tree(callee, depth + 1));
            for arg in args {
                s.push('\n');
                s.push_str(&expr_to_tree(arg, depth + 1));
            }
            s
        }
        Expr::Array(elems) => {
            let mut s = format!("{}Array", i);
            for e in elems {
                s.push('\n');
                s.push_str(&expr_to_tree(e, depth + 1));
            }
            s
        }
        Expr::Dict(pairs) => {
            let mut s = format!("{}Dict", i);
            for (k, v) in pairs {
                s.push_str(&format!(
                    "\n{}  Key\n{}\n{}  Value\n{}",
                    i,
                    expr_to_tree(k, depth + 2),
                    i,
                    expr_to_tree(v, depth + 2)
                ));
            }
            s
        }
        Expr::Index(obj, idx) => {
            format!(
                "{}Index\n{}\n{}",
                i,
                expr_to_tree(obj, depth + 1),
                expr_to_tree(idx, depth + 1)
            )
        }
        Expr::Slice(obj, start, end) => {
            let mut s = format!("{}Slice\n{}", i, expr_to_tree(obj, depth + 1));
            if let Some(st) = start {
                s.push('\n');
                s.push_str(&expr_to_tree(st, depth + 1));
            }
            if let Some(en) = end {
                s.push('\n');
                s.push_str(&expr_to_tree(en, depth + 1));
            }
            s
        }
        Expr::Spread(e) => format!("{}Spread\n{}", i, expr_to_tree(e, depth + 1)),
        Expr::Member(obj, name) => {
            format!("{}Member {:?}\n{}", i, name, expr_to_tree(obj, depth + 1))
        }
        Expr::FunctionExpr(params, body) => {
            format!(
                "{}FunctionExpr [{}]\n{}",
                i,
                params.join(", "),
                stmt_to_tree(body, depth + 1)
            )
        }
        Expr::AsyncFunctionExpr(params, body) => {
            format!(
                "{}AsyncFunctionExpr [{}]\n{}",
                i,
                params.join(", "),
                stmt_to_tree(body, depth + 1)
            )
        }
        Expr::StringInterp(parts) => {
            let mut s = format!("{}StringInterp", i);
            for p in parts {
                s.push('\n');
                s.push_str(&expr_to_tree(p, depth + 1));
            }
            s
        }
        Expr::StructInit { name, fields } => {
            let mut s = format!("{}StructInit {:?}", i, name);
            for (fname, fval) in fields {
                s.push_str(&format!(
                    "\n{}  Field {:?}\n{}",
                    i,
                    fname,
                    expr_to_tree(fval, depth + 2)
                ));
            }
            s
        }
        Expr::Await(e) => format!("{}Await\n{}", i, expr_to_tree(e, depth + 1)),
        Expr::OptionalMember(obj, name) => {
            format!(
                "{}OptionalMember {:?}\n{}",
                i,
                name,
                expr_to_tree(obj, depth + 1)
            )
        }
        Expr::OptionalCall(obj, method, args) => {
            let mut s = format!(
                "{}OptionalCall {:?}\n{}",
                i,
                method,
                expr_to_tree(obj, depth + 1)
            );
            for arg in args {
                s.push('\n');
                s.push_str(&expr_to_tree(arg, depth + 1));
            }
            s
        }
        Expr::Ternary(cond, then, else_) => {
            format!(
                "{}Ternary\n{}\n{}\n{}",
                i,
                expr_to_tree(cond, depth + 1),
                expr_to_tree(then, depth + 1),
                expr_to_tree(else_, depth + 1)
            )
        }
    }
}

// ── JSON format ───────────────────────────────────────────────────────────────

pub fn program_to_json(program: &Program) -> serde_json::Value {
    use serde_json::json;
    let stmts: Vec<_> = program
        .statements
        .iter()
        .filter(|s| !matches!(s, Stmt::Line(_)))
        .map(stmt_to_json)
        .collect();
    json!({ "type": "Program", "statements": stmts })
}

fn stmt_to_json(stmt: &Stmt) -> serde_json::Value {
    use serde_json::json;
    match stmt {
        Stmt::Expr(e) => json!({ "type": "Expr", "expr": expr_to_json(e) }),
        Stmt::Let(name, expr) => {
            json!({ "type": "Let", "name": name, "value": expr_to_json(expr) })
        }
        Stmt::Assign(target, value) => {
            json!({ "type": "Assign", "target": expr_to_json(target), "value": expr_to_json(value) })
        }
        Stmt::CompoundAssign(target, op, value) => {
            json!({ "type": "CompoundAssign", "op": format!("{:?}", op), "target": expr_to_json(target), "value": expr_to_json(value) })
        }
        Stmt::Block(stmts) => {
            let children: Vec<_> = stmts
                .iter()
                .filter(|s| !matches!(s, Stmt::Line(_)))
                .map(stmt_to_json)
                .collect();
            json!({ "type": "Block", "statements": children })
        }
        Stmt::If(cond, then, else_opt) => {
            let mut v = json!({ "type": "If", "condition": expr_to_json(cond), "then": stmt_to_json(then) });
            if let Some(e) = else_opt {
                v["else"] = stmt_to_json(e);
            }
            v
        }
        Stmt::While(cond, body) => {
            json!({ "type": "While", "condition": expr_to_json(cond), "body": stmt_to_json(body) })
        }
        Stmt::For(var, iter, body) => {
            json!({ "type": "For", "variable": var, "iterable": expr_to_json(iter), "body": stmt_to_json(body) })
        }
        Stmt::Return(Some(e)) => json!({ "type": "Return", "value": expr_to_json(e) }),
        Stmt::Return(None) => json!({ "type": "Return" }),
        Stmt::Break(label) => json!({ "type": "Break", "label": label }),
        Stmt::Continue(label) => json!({ "type": "Continue", "label": label }),
        Stmt::Labeled(label, inner) => {
            json!({ "type": "Labeled", "label": label, "body": stmt_to_json(inner) })
        }
        Stmt::Function(name, params, body) => {
            json!({ "type": "Function", "name": name, "params": params, "body": stmt_to_json(body) })
        }
        Stmt::AsyncFunction(name, params, body) => {
            json!({ "type": "AsyncFunction", "name": name, "params": params, "body": stmt_to_json(body) })
        }
        Stmt::Import(module) => json!({ "type": "Import", "module": module }),
        Stmt::ImportAs(module, alias) => {
            json!({ "type": "ImportAs", "module": module, "alias": alias })
        }
        Stmt::FromImport(module, items) => {
            json!({ "type": "FromImport", "module": module, "items": items })
        }
        Stmt::FromImportAs(module, items) => {
            let pairs: Vec<_> = items
                .iter()
                .map(|(k, v)| json!({ "name": k, "alias": v }))
                .collect();
            json!({ "type": "FromImportAs", "module": module, "items": pairs })
        }
        Stmt::TryCatch(try_body, err_var, catch_body, finally_opt) => {
            let mut v = json!({ "type": "TryCatch", "errorVar": err_var, "try": stmt_to_json(try_body), "catch": stmt_to_json(catch_body) });
            if let Some(f) = finally_opt {
                v["finally"] = stmt_to_json(f);
            }
            v
        }
        Stmt::Throw(e) => json!({ "type": "Throw", "value": expr_to_json(e) }),
        Stmt::StructDecl {
            name,
            fields,
            methods,
        } => {
            let method_list: Vec<_> = methods
                .iter()
                .map(|(mname, params, body)| {
                    json!({ "name": mname, "params": params, "body": stmt_to_json(body) })
                })
                .collect();
            json!({ "type": "StructDecl", "name": name, "fields": fields, "methods": method_list })
        }
        Stmt::EnumDecl { name, variants } => {
            let variant_list: Vec<_> = variants
                .iter()
                .map(|(vname, fields)| json!({ "name": vname, "fields": fields }))
                .collect();
            json!({ "type": "EnumDecl", "name": name, "variants": variant_list })
        }
        Stmt::Match { subject, arms } => {
            let arm_list: Vec<_> = arms
                .iter()
                .map(|(pat, body)| {
                    json!({ "pattern": format!("{:?}", pat), "body": stmt_to_json(body) })
                })
                .collect();
            json!({ "type": "Match", "subject": expr_to_json(subject), "arms": arm_list })
        }
        Stmt::LetDestructure {
            pattern,
            initializer,
        } => {
            json!({ "type": "LetDestructure", "pattern": format!("{:?}", pattern), "value": expr_to_json(initializer) })
        }
        Stmt::Line(_) => serde_json::Value::Null,
        Stmt::Debugger => json!({ "type": "Debugger" }),
    }
}

fn expr_to_json(expr: &Expr) -> serde_json::Value {
    use serde_json::json;
    match expr {
        Expr::Integer(n) => json!({ "type": "Integer", "value": n }),
        Expr::Float(f) => json!({ "type": "Float", "value": f }),
        Expr::String(s) => json!({ "type": "String", "value": s }),
        Expr::Bool(b) => json!({ "type": "Bool", "value": b }),
        Expr::Null => json!({ "type": "Null" }),
        Expr::Identifier(name) => json!({ "type": "Identifier", "name": name }),
        Expr::Binary(left, op, right) => {
            json!({ "type": "Binary", "op": format!("{:?}", op), "left": expr_to_json(left), "right": expr_to_json(right) })
        }
        Expr::Unary(op, operand) => {
            json!({ "type": "Unary", "op": format!("{:?}", op), "operand": expr_to_json(operand) })
        }
        Expr::Call(callee, args) => {
            json!({ "type": "Call", "callee": expr_to_json(callee), "args": args.iter().map(expr_to_json).collect::<Vec<_>>() })
        }
        Expr::Array(elems) => {
            json!({ "type": "Array", "elements": elems.iter().map(expr_to_json).collect::<Vec<_>>() })
        }
        Expr::Dict(pairs) => {
            let p: Vec<_> = pairs
                .iter()
                .map(|(k, v)| json!({ "key": expr_to_json(k), "value": expr_to_json(v) }))
                .collect();
            json!({ "type": "Dict", "pairs": p })
        }
        Expr::Index(obj, idx) => {
            json!({ "type": "Index", "object": expr_to_json(obj), "index": expr_to_json(idx) })
        }
        Expr::Slice(obj, start, end) => {
            json!({ "type": "Slice", "object": expr_to_json(obj), "start": start.as_ref().map(|e| expr_to_json(e)), "end": end.as_ref().map(|e| expr_to_json(e)) })
        }
        Expr::Spread(e) => json!({ "type": "Spread", "expr": expr_to_json(e) }),
        Expr::Member(obj, name) => {
            json!({ "type": "Member", "object": expr_to_json(obj), "member": name })
        }
        Expr::FunctionExpr(params, body) => {
            json!({ "type": "FunctionExpr", "params": params, "body": stmt_to_json(body) })
        }
        Expr::AsyncFunctionExpr(params, body) => {
            json!({ "type": "AsyncFunctionExpr", "params": params, "body": stmt_to_json(body) })
        }
        Expr::StringInterp(parts) => {
            json!({ "type": "StringInterp", "parts": parts.iter().map(expr_to_json).collect::<Vec<_>>() })
        }
        Expr::StructInit { name, fields } => {
            let field_list: Vec<_> = fields
                .iter()
                .map(|(fname, fval)| json!({ "name": fname, "value": expr_to_json(fval) }))
                .collect();
            json!({ "type": "StructInit", "name": name, "fields": field_list })
        }
        Expr::Await(e) => json!({ "type": "Await", "expr": expr_to_json(e) }),
        Expr::OptionalMember(obj, name) => {
            json!({ "type": "OptionalMember", "object": expr_to_json(obj), "member": name })
        }
        Expr::OptionalCall(obj, method, args) => {
            json!({ "type": "OptionalCall", "object": expr_to_json(obj), "method": method, "args": args.iter().map(expr_to_json).collect::<Vec<_>>() })
        }
        Expr::Ternary(cond, then, else_) => {
            json!({ "type": "Ternary", "condition": expr_to_json(cond), "then": expr_to_json(then), "else": expr_to_json(else_) })
        }
    }
}
