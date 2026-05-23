use aether_lang::checker::check;
use aether_lang::lexer::Scanner;
use aether_lang::parser::Parser;

fn diagnostics(src: &str) -> Vec<String> {
    let mut scanner = Scanner::new(src);
    let tokens = scanner.scan_tokens().expect("scan");
    let mut parser = Parser::new(tokens);
    let program = parser.parse().expect("parse");
    check(&program).into_iter().map(|d| d.message).collect()
}

fn is_clean(src: &str) -> bool {
    diagnostics(src).is_empty()
}

// ── undefined variable detection ─────────────────────────────────────────────

#[test]
fn test_check_clean_let() {
    assert!(is_clean("let x = 1\n"));
}

#[test]
fn test_check_undefined_use() {
    let diags = diagnostics("let y = x\n");
    assert!(
        diags.iter().any(|d| d.contains("undefined variable 'x'")),
        "got: {:?}",
        diags
    );
}

#[test]
fn test_check_let_before_use_ok() {
    assert!(is_clean("let x = 1\nlet y = x\n"));
}

#[test]
fn test_check_fn_params_in_scope() {
    assert!(is_clean("fn add(a, b) {\n    return a + b\n}\n"));
}

#[test]
fn test_check_undefined_in_fn_body() {
    let diags = diagnostics("fn f() {\n    return z\n}\n");
    assert!(
        diags.iter().any(|d| d.contains("undefined variable 'z'")),
        "got: {:?}",
        diags
    );
}

#[test]
fn test_check_for_loop_var_in_scope() {
    assert!(is_clean(
        "let arr = [1, 2]\nfor x in arr {\n    println(x)\n}\n"
    ));
}

#[test]
fn test_check_undefined_in_while_cond() {
    let diags = diagnostics("while (counter < 10) {\n    println(1)\n}\n");
    assert!(
        diags.iter().any(|d| d.contains("'counter'")),
        "got: {:?}",
        diags
    );
}

#[test]
fn test_check_try_catch_err_var_in_scope() {
    assert!(is_clean(
        "try {\n    println(1)\n} catch(e) {\n    println(e)\n}\n"
    ));
}

#[test]
fn test_check_struct_self_in_method() {
    assert!(is_clean(
        "struct Point {\n    x\n    fn get_x() {\n        return self.x\n    }\n}\n"
    ));
}

#[test]
fn test_check_forward_reference_fn_ok() {
    // Functions at the top level should be hoisted
    assert!(is_clean(
        "fn main() {\n    helper()\n}\nfn helper() {\n    println(1)\n}\n"
    ));
}

#[test]
fn test_check_builtin_names_always_defined() {
    assert!(is_clean("println(len([1, 2]))\n"));
}

#[test]
fn test_check_stdlib_names_defined() {
    assert!(is_clean("let xs = map([1, 2], fn(x) { return x + 1 })\n"));
}

#[test]
fn test_check_import_binds_name() {
    assert!(is_clean("import math\nlet x = math.pi\n"));
}

#[test]
fn test_check_from_import_binds_names() {
    assert!(is_clean("from testing import assert_eq\nassert_eq(1, 1)\n"));
}

#[test]
fn test_check_function_expr_params_in_scope() {
    assert!(is_clean("let f = fn(a) { return a }\n"));
}

#[test]
fn test_check_function_expr_undefined() {
    let diags = diagnostics("let f = fn() { return no_var }\n");
    assert!(
        diags.iter().any(|d| d.contains("'no_var'")),
        "got: {:?}",
        diags
    );
}

#[test]
fn test_check_closure_captures_outer() {
    assert!(is_clean("let x = 10\nlet f = fn() { return x }\n"));
}

#[test]
fn test_check_match_bind_in_scope() {
    assert!(is_clean("let v = 1\nmatch v {\n    x => println(x)\n}\n"));
}

#[test]
fn test_check_enum_name_defined() {
    assert!(is_clean("enum Color {\n    Red\n    Green\n}\n"));
}

#[test]
fn test_check_struct_name_defined() {
    assert!(is_clean(
        "struct Point {\n    x\n    y\n}\nlet p = Point { x: 1, y: 2 }\n"
    ));
}

#[test]
fn test_check_if_cond_scope() {
    let diags = diagnostics("if (z > 0) {\n    println(1)\n}\n");
    assert!(diags.iter().any(|d| d.contains("'z'")), "got: {:?}", diags);
}

#[test]
fn test_check_string_interp_undefined() {
    let diags = diagnostics("let s = \"hello ${no_var}\"\n");
    assert!(
        diags.iter().any(|d| d.contains("'no_var'")),
        "got: {:?}",
        diags
    );
}

#[test]
fn test_check_member_access_only_checks_object() {
    // obj.field — only 'obj' is checked, not 'field'
    assert!(is_clean("let obj = {}\nlet v = obj.missing\n"));
}

#[test]
fn test_check_ternary_checks_all_parts() {
    let diags = diagnostics("let x = a > 0 ? 1 : 0\n");
    assert!(diags.iter().any(|d| d.contains("'a'")), "got: {:?}", diags);
}

#[test]
fn test_check_destructure_array_binds_names() {
    assert!(is_clean("let [a, b] = [1, 2]\nlet s = a + b\n"));
}

#[test]
fn test_check_destructure_dict_binds_names() {
    assert!(is_clean(
        "let {x, y} = {\"x\": 1, \"y\": 2}\nlet s = x + y\n"
    ));
}

#[test]
fn test_check_promise_is_defined() {
    assert!(is_clean("let p = Promise.all([])\n"));
}
