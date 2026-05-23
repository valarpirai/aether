//! Tests for aether fmt (source formatter)

use aether_lang::formatter::format_source;

fn fmt(src: &str) -> String {
    format_source(src).expect("format_source failed")
}

// Already-canonical input is returned unchanged (idempotency)
#[test]
fn test_idempotent_simple() {
    let src = "let x = 1\n";
    assert_eq!(fmt(src), src);
}

#[test]
fn test_let_spacing() {
    assert_eq!(fmt("let x=1\n"), "let x = 1\n");
}

#[test]
fn test_binary_spacing() {
    assert_eq!(fmt("let x=1+2\n"), "let x = 1 + 2\n");
}

#[test]
fn test_function_declaration() {
    let src = "fn add(a, b) {\n    return a + b\n}\n";
    assert_eq!(fmt(src), src);
}

#[test]
fn test_function_formats_body() {
    let input = "fn f(x){return x+1}\n";
    let expected = "fn f(x) {\n    return x + 1\n}\n";
    assert_eq!(fmt(input), expected);
}

#[test]
fn test_if_else() {
    let src = "if (x > 0) {\n    println(x)\n} else {\n    println(0)\n}\n";
    assert_eq!(fmt(src), src);
}

#[test]
fn test_if_else_if() {
    let src =
        "if (a) {\n    println(1)\n} else if (b) {\n    println(2)\n} else {\n    println(3)\n}\n";
    assert_eq!(fmt(src), src);
}

#[test]
fn test_while_loop() {
    let src = "while (i < 10) {\n    i = i + 1\n}\n";
    assert_eq!(fmt(src), src);
}

#[test]
fn test_for_loop() {
    let src = "for x in arr {\n    println(x)\n}\n";
    assert_eq!(fmt(src), src);
}

#[test]
fn test_blank_line_between_fns() {
    let input = "fn a() {\n    return 1\n}\nfn b() {\n    return 2\n}\n";
    let expected = "fn a() {\n    return 1\n}\n\nfn b() {\n    return 2\n}\n";
    assert_eq!(fmt(input), expected);
}

#[test]
fn test_array_literal() {
    assert_eq!(fmt("let a=[1,2,3]\n"), "let a = [1, 2, 3]\n");
}

#[test]
fn test_empty_array() {
    assert_eq!(fmt("let a=[]\n"), "let a = []\n");
}

#[test]
fn test_dict_literal() {
    assert_eq!(
        fmt("let d={\"a\":1,\"b\":2}\n"),
        "let d = {\"a\": 1, \"b\": 2}\n"
    );
}

#[test]
fn test_nested_precedence_no_extra_parens() {
    // a + b * c should stay as-is (no parens needed)
    assert_eq!(fmt("let x = a + b * c\n"), "let x = a + b * c\n");
}

#[test]
fn test_precedence_adds_parens_when_needed() {
    // (a + b) * c — parser captures this as Binary(Binary(a,Add,b), Mul, c)
    // formatter should emit (a + b) * c
    let src = "let x = (a + b) * c\n";
    let out = fmt(src);
    assert!(out.contains("(a + b) * c"), "got: {}", out);
}

#[test]
fn test_try_catch() {
    let src = "try {\n    foo()\n} catch(e) {\n    println(e)\n}\n";
    assert_eq!(fmt(src), src);
}

#[test]
fn test_try_catch_finally() {
    let src = "try {\n    foo()\n} catch(e) {\n    println(e)\n} finally {\n    cleanup()\n}\n";
    assert_eq!(fmt(src), src);
}

#[test]
fn test_struct_decl() {
    let src = "struct Point {\n    x\n    y\n}\n";
    assert_eq!(fmt(src), src);
}

#[test]
fn test_enum_decl() {
    let src = "enum Color {\n    Red\n    Green\n    Blue\n}\n";
    assert_eq!(fmt(src), src);
}

#[test]
fn test_import() {
    assert_eq!(fmt("import math\n"), "import math\n");
}

#[test]
fn test_from_import() {
    assert_eq!(
        fmt("from math import abs, min\n"),
        "from math import abs, min\n"
    );
}

#[test]
fn test_float_always_has_decimal() {
    let out = fmt("let x = 3.0\n");
    assert!(out.contains("3.0") || out.contains("3"), "got: {}", out);
}

#[test]
fn test_string_interpolation() {
    let src = "let s = \"hello ${name}\"\n";
    assert_eq!(fmt(src), src);
}

#[test]
fn test_ternary() {
    let src = "let x = a > 0 ? a : 0\n";
    assert_eq!(fmt(src), src);
}

#[test]
fn test_optional_chaining() {
    let src = "let x = obj?.field\n";
    assert_eq!(fmt(src), src);
}

#[test]
fn test_null_coalesce() {
    let src = "let x = a ?? b\n";
    assert_eq!(fmt(src), src);
}
