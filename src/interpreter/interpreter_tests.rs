//! Tests for the interpreter module

use super::environment::{Environment, RuntimeError};
use super::value::Value;

// Value tests
#[test]
fn test_value_creation() {
    let int_val = Value::Int(42);
    let float_val = Value::Float(3.14);
    let string_val = Value::string("hello");
    let bool_val = Value::Bool(true);
    let null_val = Value::Null;

    assert_eq!(int_val, Value::Int(42));
    assert_eq!(float_val, Value::Float(3.14));
    assert_eq!(string_val, Value::string("hello"));
    assert_eq!(bool_val, Value::Bool(true));
    assert_eq!(null_val, Value::Null);
}

#[test]
fn test_value_display() {
    assert_eq!(format!("{}", Value::Int(42)), "42");
    assert_eq!(format!("{}", Value::Float(3.14)), "3.14");
    assert_eq!(format!("{}", Value::string("hello")), "hello");
    assert_eq!(format!("{}", Value::Bool(true)), "true");
    assert_eq!(format!("{}", Value::Null), "null");
    assert_eq!(
        format!("{}", Value::array(vec![Value::Int(1), Value::Int(2)])),
        "[1, 2]"
    );
}

#[test]
fn test_value_is_truthy() {
    assert!(Value::Bool(true).is_truthy());
    assert!(!Value::Bool(false).is_truthy());
    assert!(!Value::Null.is_truthy());
    assert!(!Value::Int(0).is_truthy());
    assert!(Value::Int(1).is_truthy());
    assert!(!Value::Float(0.0).is_truthy());
    assert!(Value::Float(1.0).is_truthy());
    assert!(!Value::string("").is_truthy());
    assert!(Value::string("hello").is_truthy());
    assert!(!Value::array(vec![]).is_truthy());
    assert!(Value::array(vec![Value::Int(1)]).is_truthy());
}

#[test]
fn test_value_type_name() {
    assert_eq!(Value::Int(42).type_name(), "int");
    assert_eq!(Value::Float(3.14).type_name(), "float");
    assert_eq!(Value::string("hello").type_name(), "string");
    assert_eq!(Value::Bool(true).type_name(), "bool");
    assert_eq!(Value::Null.type_name(), "null");
    assert_eq!(Value::array(vec![]).type_name(), "array");
}

// Environment tests
#[test]
fn test_environment_define_and_get() {
    let mut env = Environment::new();
    env.define("x".to_string(), Value::Int(42));

    let value = env.get("x").unwrap();
    assert_eq!(value, Value::Int(42));
}

#[test]
fn test_environment_undefined_variable() {
    let env = Environment::new();
    let result = env.get("x");
    assert!(matches!(result, Err(RuntimeError::UndefinedVariable(_))));
}

#[test]
fn test_environment_set_existing() {
    let mut env = Environment::new();
    env.define("x".to_string(), Value::Int(42));
    env.set("x", Value::Int(100)).unwrap();

    let value = env.get("x").unwrap();
    assert_eq!(value, Value::Int(100));
}

#[test]
fn test_environment_set_undefined() {
    let mut env = Environment::new();
    let result = env.set("x", Value::Int(42));
    assert!(matches!(result, Err(RuntimeError::UndefinedVariable(_))));
}

#[test]
fn test_environment_nested_scopes() {
    let mut global = Environment::new();
    global.define("x".to_string(), Value::Int(10));

    let mut local = Environment::with_parent(global.clone());
    local.define("y".to_string(), Value::Int(20));

    // Can access both local and parent variables
    assert_eq!(local.get("y").unwrap(), Value::Int(20));
    assert_eq!(local.get("x").unwrap(), Value::Int(10));

    // Parent cannot access child variables
    assert!(global.get("y").is_err());
}

#[test]
fn test_environment_shadowing() {
    let mut global = Environment::new();
    global.define("x".to_string(), Value::Int(10));

    let mut local = Environment::with_parent(global.clone());
    local.define("x".to_string(), Value::Int(20));

    // Local shadows global
    assert_eq!(local.get("x").unwrap(), Value::Int(20));
    assert_eq!(global.get("x").unwrap(), Value::Int(10));
}

// Evaluator tests
use super::evaluator::Evaluator;
use crate::parser::ast::{BinaryOp, Expr, UnaryOp};

#[test]
fn test_eval_literals() {
    let mut eval = Evaluator::new();

    assert_eq!(eval.eval_expr(&Expr::Integer(42)).unwrap(), Value::Int(42));
    assert_eq!(eval.eval_expr(&Expr::Float(3.14)).unwrap(), Value::Float(3.14));
    assert_eq!(
        eval.eval_expr(&Expr::String("hello".to_string())).unwrap(),
        Value::string("hello")
    );
    assert_eq!(eval.eval_expr(&Expr::Bool(true)).unwrap(), Value::Bool(true));
    assert_eq!(eval.eval_expr(&Expr::Null).unwrap(), Value::Null);
}

#[test]
fn test_eval_arithmetic() {
    let mut eval = Evaluator::new();

    // Addition
    let expr = Expr::Binary(
        Box::new(Expr::Integer(10)),
        BinaryOp::Add,
        Box::new(Expr::Integer(20)),
    );
    assert_eq!(eval.eval_expr(&expr).unwrap(), Value::Int(30));

    // Subtraction
    let expr = Expr::Binary(
        Box::new(Expr::Integer(30)),
        BinaryOp::Subtract,
        Box::new(Expr::Integer(10)),
    );
    assert_eq!(eval.eval_expr(&expr).unwrap(), Value::Int(20));

    // Multiplication
    let expr = Expr::Binary(
        Box::new(Expr::Integer(6)),
        BinaryOp::Multiply,
        Box::new(Expr::Integer(7)),
    );
    assert_eq!(eval.eval_expr(&expr).unwrap(), Value::Int(42));

    // Division
    let expr = Expr::Binary(
        Box::new(Expr::Integer(20)),
        BinaryOp::Divide,
        Box::new(Expr::Integer(4)),
    );
    assert_eq!(eval.eval_expr(&expr).unwrap(), Value::Int(5));
}

#[test]
fn test_eval_unary() {
    let mut eval = Evaluator::new();

    // Negation
    let expr = Expr::Unary(UnaryOp::Negate, Box::new(Expr::Integer(42)));
    assert_eq!(eval.eval_expr(&expr).unwrap(), Value::Int(-42));

    // Not
    let expr = Expr::Unary(UnaryOp::Not, Box::new(Expr::Bool(true)));
    assert_eq!(eval.eval_expr(&expr).unwrap(), Value::Bool(false));
}

#[test]
fn test_eval_comparison() {
    let mut eval = Evaluator::new();

    // Less than
    let expr = Expr::Binary(
        Box::new(Expr::Integer(10)),
        BinaryOp::Less,
        Box::new(Expr::Integer(20)),
    );
    assert_eq!(eval.eval_expr(&expr).unwrap(), Value::Bool(true));

    // Greater than
    let expr = Expr::Binary(
        Box::new(Expr::Integer(30)),
        BinaryOp::Greater,
        Box::new(Expr::Integer(20)),
    );
    assert_eq!(eval.eval_expr(&expr).unwrap(), Value::Bool(true));

    // Equal
    let expr = Expr::Binary(
        Box::new(Expr::Integer(42)),
        BinaryOp::Equal,
        Box::new(Expr::Integer(42)),
    );
    assert_eq!(eval.eval_expr(&expr).unwrap(), Value::Bool(true));
}

#[test]
fn test_eval_logical() {
    let mut eval = Evaluator::new();

    // And (short-circuit)
    let expr = Expr::Binary(
        Box::new(Expr::Bool(true)),
        BinaryOp::And,
        Box::new(Expr::Bool(false)),
    );
    assert_eq!(eval.eval_expr(&expr).unwrap(), Value::Bool(false));

    // Or (short-circuit)
    let expr = Expr::Binary(
        Box::new(Expr::Bool(true)),
        BinaryOp::Or,
        Box::new(Expr::Bool(false)),
    );
    assert_eq!(eval.eval_expr(&expr).unwrap(), Value::Bool(true));
}

#[test]
fn test_eval_string_concat() {
    let mut eval = Evaluator::new();

    let expr = Expr::Binary(
        Box::new(Expr::String("Hello, ".to_string())),
        BinaryOp::Add,
        Box::new(Expr::String("World!".to_string())),
    );
    assert_eq!(
        eval.eval_expr(&expr).unwrap(),
        Value::string("Hello, World!")
    );
}

#[test]
fn test_eval_array_literal() {
    let mut eval = Evaluator::new();

    let expr = Expr::Array(vec![
        Expr::Integer(1),
        Expr::Integer(2),
        Expr::Integer(3),
    ]);
    assert_eq!(
        eval.eval_expr(&expr).unwrap(),
        Value::array(vec![Value::Int(1), Value::Int(2), Value::Int(3)])
    );
}

#[test]
fn test_eval_array_indexing() {
    let mut eval = Evaluator::new();
    eval.environment.define(
        "arr".to_string(),
        Value::array(vec![Value::Int(10), Value::Int(20), Value::Int(30)]),
    );

    let expr = Expr::Index(
        Box::new(Expr::Identifier("arr".to_string())),
        Box::new(Expr::Integer(1)),
    );
    assert_eq!(eval.eval_expr(&expr).unwrap(), Value::Int(20));
}

#[test]
fn test_eval_division_by_zero() {
    let mut eval = Evaluator::new();

    let expr = Expr::Binary(
        Box::new(Expr::Integer(10)),
        BinaryOp::Divide,
        Box::new(Expr::Integer(0)),
    );
    assert!(matches!(
        eval.eval_expr(&expr),
        Err(RuntimeError::DivisionByZero)
    ));
}

#[test]
fn test_eval_index_out_of_bounds() {
    let mut eval = Evaluator::new();
    eval.environment.define(
        "arr".to_string(),
        Value::array(vec![Value::Int(10)]),
    );

    let expr = Expr::Index(
        Box::new(Expr::Identifier("arr".to_string())),
        Box::new(Expr::Integer(5)),
    );
    assert!(matches!(
        eval.eval_expr(&expr),
        Err(RuntimeError::IndexOutOfBounds { .. })
    ));
}

// Statement execution tests
use crate::parser::ast::Stmt;

#[test]
fn test_exec_let_statement() {
    let mut eval = Evaluator::new();

    let stmt = Stmt::Let("x".to_string(), Expr::Integer(42));
    eval.exec_stmt(&stmt).unwrap();

    assert_eq!(eval.environment.get("x").unwrap(), Value::Int(42));
}

#[test]
fn test_exec_assignment() {
    let mut eval = Evaluator::new();

    eval.environment.define("x".to_string(), Value::Int(10));

    let stmt = Stmt::Assign(Expr::Identifier("x".to_string()), Expr::Integer(42));
    eval.exec_stmt(&stmt).unwrap();

    assert_eq!(eval.environment.get("x").unwrap(), Value::Int(42));
}

#[test]
fn test_exec_compound_assignment() {
    let mut eval = Evaluator::new();

    eval.environment.define("x".to_string(), Value::Int(10));

    let stmt = Stmt::CompoundAssign(
        Expr::Identifier("x".to_string()),
        BinaryOp::Add,
        Expr::Integer(5),
    );
    eval.exec_stmt(&stmt).unwrap();

    assert_eq!(eval.environment.get("x").unwrap(), Value::Int(15));
}

#[test]
fn test_exec_block() {
    let mut eval = Evaluator::new();

    let stmt = Stmt::Block(vec![
        Stmt::Let("x".to_string(), Expr::Integer(10)),
        Stmt::Let("y".to_string(), Expr::Integer(20)),
    ]);
    eval.exec_stmt(&stmt).unwrap();

    // Variables are in outer scope after block
    assert_eq!(eval.environment.get("x").unwrap(), Value::Int(10));
    assert_eq!(eval.environment.get("y").unwrap(), Value::Int(20));
}

#[test]
fn test_exec_if_true() {
    let mut eval = Evaluator::new();

    let stmt = Stmt::If(
        Expr::Bool(true),
        Box::new(Stmt::Let("x".to_string(), Expr::Integer(42))),
        None,
    );
    eval.exec_stmt(&stmt).unwrap();

    assert_eq!(eval.environment.get("x").unwrap(), Value::Int(42));
}

#[test]
fn test_exec_if_false() {
    let mut eval = Evaluator::new();

    let stmt = Stmt::If(
        Expr::Bool(false),
        Box::new(Stmt::Let("x".to_string(), Expr::Integer(42))),
        None,
    );
    eval.exec_stmt(&stmt).unwrap();

    // x should not be defined
    assert!(eval.environment.get("x").is_err());
}

#[test]
fn test_exec_if_else() {
    let mut eval = Evaluator::new();

    let stmt = Stmt::If(
        Expr::Bool(false),
        Box::new(Stmt::Let("x".to_string(), Expr::Integer(42))),
        Some(Box::new(Stmt::Let("x".to_string(), Expr::Integer(100)))),
    );
    eval.exec_stmt(&stmt).unwrap();

    assert_eq!(eval.environment.get("x").unwrap(), Value::Int(100));
}

#[test]
fn test_exec_while_loop() {
    let mut eval = Evaluator::new();

    eval.environment.define("i".to_string(), Value::Int(0));
    eval.environment.define("sum".to_string(), Value::Int(0));

    // while (i < 5) { sum += i; i += 1 }
    let stmt = Stmt::While(
        Expr::Binary(
            Box::new(Expr::Identifier("i".to_string())),
            BinaryOp::Less,
            Box::new(Expr::Integer(5)),
        ),
        Box::new(Stmt::Block(vec![
            Stmt::CompoundAssign(
                Expr::Identifier("sum".to_string()),
                BinaryOp::Add,
                Expr::Identifier("i".to_string()),
            ),
            Stmt::CompoundAssign(
                Expr::Identifier("i".to_string()),
                BinaryOp::Add,
                Expr::Integer(1),
            ),
        ])),
    );
    eval.exec_stmt(&stmt).unwrap();

    assert_eq!(eval.environment.get("sum").unwrap(), Value::Int(10)); // 0+1+2+3+4
}

#[test]
fn test_exec_for_loop() {
    let mut eval = Evaluator::new();

    eval.environment.define("sum".to_string(), Value::Int(0));

    // for i in [1, 2, 3] { sum += i }
    let stmt = Stmt::For(
        "i".to_string(),
        Expr::Array(vec![Expr::Integer(1), Expr::Integer(2), Expr::Integer(3)]),
        Box::new(Stmt::CompoundAssign(
            Expr::Identifier("sum".to_string()),
            BinaryOp::Add,
            Expr::Identifier("i".to_string()),
        )),
    );
    eval.exec_stmt(&stmt).unwrap();

    assert_eq!(eval.environment.get("sum").unwrap(), Value::Int(6));
}

// Function tests
#[test]
fn test_function_declaration() {
    let mut eval = Evaluator::new();

    let stmt = Stmt::Function(
        "add".to_string(),
        vec!["a".to_string(), "b".to_string()],
        Box::new(Stmt::Return(Some(Expr::Binary(
            Box::new(Expr::Identifier("a".to_string())),
            BinaryOp::Add,
            Box::new(Expr::Identifier("b".to_string())),
        )))),
    );
    eval.exec_stmt(&stmt).unwrap();

    // Function should be defined
    assert!(matches!(
        eval.environment.get("add").unwrap(),
        Value::Function { .. }
    ));
}

#[test]
fn test_function_call() {
    let mut eval = Evaluator::new();

    // Define function: fn add(a, b) { return a + b }
    let func_stmt = Stmt::Function(
        "add".to_string(),
        vec!["a".to_string(), "b".to_string()],
        Box::new(Stmt::Return(Some(Expr::Binary(
            Box::new(Expr::Identifier("a".to_string())),
            BinaryOp::Add,
            Box::new(Expr::Identifier("b".to_string())),
        )))),
    );
    eval.exec_stmt(&func_stmt).unwrap();

    // Call function: add(10, 20)
    let call_expr = Expr::Call(
        Box::new(Expr::Identifier("add".to_string())),
        vec![Expr::Integer(10), Expr::Integer(20)],
    );
    let result = eval.eval_expr(&call_expr).unwrap();

    assert_eq!(result, Value::Int(30));
}
