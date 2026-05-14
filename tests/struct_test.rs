//! Tests for struct declarations and instances

use aether_lang::interpreter::Evaluator;
use aether_lang::lexer::Scanner;
use aether_lang::parser::Parser;

fn eval(source: &str) -> Result<String, String> {
    let mut scanner = Scanner::new(source);
    let tokens = scanner.scan_tokens().map_err(|e| e.to_string())?;
    let mut parser = Parser::new(tokens);
    let program = parser.parse().map_err(|e| e.to_string())?;
    let mut evaluator = Evaluator::new_without_stdlib();

    for stmt in &program.statements[..program.statements.len().saturating_sub(1)] {
        evaluator.exec_stmt(stmt).map_err(|e| e.to_string())?;
    }

    if let Some(last) = program.statements.last() {
        if let aether_lang::parser::ast::Stmt::Expr(expr) = last {
            let value = evaluator.eval_expr(expr).map_err(|e| e.to_string())?;
            return Ok(format!("{}", value));
        }
        evaluator.exec_stmt(last).map_err(|e| e.to_string())?;
    }

    Ok("null".to_string())
}

// --- struct declaration and instantiation ---

#[test]
fn test_struct_declare_and_instantiate() {
    let result = eval(
        r#"
        struct Point { x, y }
        let p = Point { x: 1, y: 2 }
        p.x
        "#,
    );
    assert_eq!(result.unwrap(), "1");
}

#[test]
fn test_struct_field_y() {
    let result = eval(
        r#"
        struct Point { x, y }
        let p = Point { x: 3, y: 7 }
        p.y
        "#,
    );
    assert_eq!(result.unwrap(), "7");
}

#[test]
fn test_struct_display() {
    let result = eval(
        r#"
        struct Point { x, y }
        let p = Point { x: 1, y: 2 }
        str(p)
        "#,
    );
    assert_eq!(result.unwrap(), "Point { x: 1, y: 2 }");
}

#[test]
fn test_struct_type_name() {
    let result = eval(
        r#"
        struct Point { x, y }
        let p = Point { x: 0, y: 0 }
        type(p)
        "#,
    );
    assert_eq!(result.unwrap(), "Point");
}

// --- field mutation ---

#[test]
fn test_struct_field_assignment() {
    let result = eval(
        r#"
        struct Point { x, y }
        let p = Point { x: 1, y: 2 }
        p.x = 99
        p.x
        "#,
    );
    assert_eq!(result.unwrap(), "99");
}

#[test]
fn test_struct_field_mutation_independent() {
    let result = eval(
        r#"
        struct Point { x, y }
        let p = Point { x: 1, y: 2 }
        p.x = 10
        p.y
        "#,
    );
    assert_eq!(result.unwrap(), "2");
}

// --- default null for unset fields ---

#[test]
fn test_struct_unset_field_is_null() {
    let result = eval(
        r#"
        struct Point { x, y }
        let p = Point { x: 5 }
        p.y
        "#,
    );
    assert_eq!(result.unwrap(), "null");
}

// --- methods ---

#[test]
fn test_struct_method_call() {
    let result = eval(
        r#"
        struct Counter {
            count
            fn increment(self) {
                self.count = self.count + 1
            }
        }
        let c = Counter { count: 0 }
        c.increment()
        c.count
        "#,
    );
    assert_eq!(result.unwrap(), "1");
}

#[test]
fn test_struct_method_with_return() {
    let result = eval(
        r#"
        struct Rectangle {
            width
            height
            fn area(self) {
                return self.width * self.height
            }
        }
        let r = Rectangle { width: 3, height: 4 }
        r.area()
        "#,
    );
    assert_eq!(result.unwrap(), "12");
}

#[test]
fn test_struct_method_with_param() {
    let result = eval(
        r#"
        struct Adder {
            base
            fn add(self, n) {
                return self.base + n
            }
        }
        let a = Adder { base: 10 }
        a.add(5)
        "#,
    );
    assert_eq!(result.unwrap(), "15");
}

// --- multiple instances ---

#[test]
fn test_struct_multiple_instances_independent() {
    let result = eval(
        r#"
        struct Point { x, y }
        let p1 = Point { x: 1, y: 2 }
        let p2 = Point { x: 10, y: 20 }
        p1.x = 99
        p2.x
        "#,
    );
    assert_eq!(result.unwrap(), "10");
}

// --- nested structs ---

#[test]
fn test_struct_nested() {
    let result = eval(
        r#"
        struct Point { x, y }
        struct Line { start, end }
        let p1 = Point { x: 0, y: 0 }
        let p2 = Point { x: 5, y: 5 }
        let line = Line { start: p1, end: p2 }
        line.end.x
        "#,
    );
    assert_eq!(result.unwrap(), "5");
}

// --- error cases ---

#[test]
fn test_struct_undefined_field_errors() {
    let result = eval(
        r#"
        struct Point { x, y }
        let p = Point { x: 1, y: 2 }
        p.z
        "#,
    );
    assert!(result.is_err());
}

#[test]
fn test_struct_undefined_method_errors() {
    let result = eval(
        r#"
        struct Point { x, y }
        let p = Point { x: 1, y: 2 }
        p.move()
        "#,
    );
    assert!(result.is_err());
}

// --- struct equality: == is identity ---

#[test]
fn test_struct_eq_same_object_is_true() {
    let result = eval(
        r#"
        struct Point { x, y }
        let a = Point { x: 1, y: 2 }
        let b = a
        a == b
        "#,
    )
    .unwrap();
    assert_eq!(result, "true");
}

#[test]
fn test_struct_eq_different_objects_is_false() {
    let result = eval(
        r#"
        struct Point { x, y }
        let a = Point { x: 1, y: 2 }
        let b = Point { x: 1, y: 2 }
        a == b
        "#,
    )
    .unwrap();
    assert_eq!(result, "false");
}

#[test]
fn test_struct_neq_same_object_is_false() {
    let result = eval(
        r#"
        struct Point { x, y }
        let a = Point { x: 1, y: 2 }
        let b = a
        a != b
        "#,
    )
    .unwrap();
    assert_eq!(result, "false");
}

// --- struct.equals() is structural depth-1 ---

#[test]
fn test_struct_equals_same_values() {
    let result = eval(
        r#"
        struct Point { x, y }
        let a = Point { x: 1, y: 2 }
        let b = Point { x: 1, y: 2 }
        a.equals(b)
        "#,
    )
    .unwrap();
    assert_eq!(result, "true");
}

#[test]
fn test_struct_equals_different_values() {
    let result = eval(
        r#"
        struct Point { x, y }
        let a = Point { x: 1, y: 2 }
        let b = Point { x: 1, y: 99 }
        a.equals(b)
        "#,
    )
    .unwrap();
    assert_eq!(result, "false");
}

#[test]
fn test_struct_equals_different_types() {
    let result = eval(
        r#"
        struct Point { x, y }
        struct Vec2 { x, y }
        let a = Point { x: 1, y: 2 }
        let b = Vec2 { x: 1, y: 2 }
        a.equals(b)
        "#,
    )
    .unwrap();
    assert_eq!(result, "false");
}

#[test]
fn test_struct_equals_same_object() {
    let result = eval(
        r#"
        struct Point { x, y }
        let a = Point { x: 1, y: 2 }
        a.equals(a)
        "#,
    )
    .unwrap();
    assert_eq!(result, "true");
}

#[test]
fn test_struct_equals_nested_struct_uses_identity() {
    // nested struct fields use == (identity), so two separate inner structs → false
    let result = eval(
        r#"
        struct Inner { v }
        struct Outer { inner }
        let a = Outer { inner: Inner { v: 1 } }
        let b = Outer { inner: Inner { v: 1 } }
        a.equals(b)
        "#,
    )
    .unwrap();
    assert_eq!(result, "false");
}

#[test]
fn test_struct_equals_nested_struct_shared_ref() {
    // same inner object shared → equals returns true
    let result = eval(
        r#"
        struct Inner { v }
        struct Outer { inner }
        let shared = Inner { v: 1 }
        let a = Outer { inner: shared }
        let b = Outer { inner: shared }
        a.equals(b)
        "#,
    )
    .unwrap();
    assert_eq!(result, "true");
}

#[test]
fn test_struct_equals_array_field_uses_identity() {
    // array fields use == (identity) — separate arrays → false
    let result = eval(
        r#"
        struct Container { items }
        let a = Container { items: [1, 2, 3] }
        let b = Container { items: [1, 2, 3] }
        a.equals(b)
        "#,
    )
    .unwrap();
    assert_eq!(result, "false");
}

#[test]
fn test_struct_equals_shared_array_field() {
    // same array reference → equals returns true
    let result = eval(
        r#"
        struct Container { items }
        let arr = [1, 2, 3]
        let a = Container { items: arr }
        let b = Container { items: arr }
        a.equals(b)
        "#,
    )
    .unwrap();
    assert_eq!(result, "true");
}
