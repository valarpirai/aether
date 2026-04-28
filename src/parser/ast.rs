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
}

/// Unary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Negate,
    Not,
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
    /// Line number marker — injected by the parser, updates evaluator's current_line
    Line(usize),
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
