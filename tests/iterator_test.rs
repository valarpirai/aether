//! Tests for the iterator protocol

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

// --- Array iterator: next() ---

#[test]
fn test_array_iterator_first_next() {
    assert_eq!(eval("[10, 20, 30].iterator().next()").unwrap(), "10");
}

#[test]
fn test_array_iterator_exhausted_returns_null() {
    let src = r#"
let it = [1].iterator()
it.next()
it.next()
"#;
    assert_eq!(eval(src).unwrap(), "null");
}

#[test]
fn test_array_iterator_has_next_true() {
    assert_eq!(eval("[1, 2].iterator().has_next()").unwrap(), "true");
}

#[test]
fn test_array_iterator_has_next_false_when_empty() {
    assert_eq!(eval("[].iterator().has_next()").unwrap(), "false");
}

#[test]
fn test_array_iterator_has_next_false_after_drain() {
    let src = r#"
let it = [42].iterator()
it.next()
it.has_next()
"#;
    assert_eq!(eval(src).unwrap(), "false");
}

#[test]
fn test_array_iterator_independent_copies() {
    let src = r#"
let arr = [1, 2, 3]
let it1 = arr.iterator()
let it2 = arr.iterator()
it1.next()
it2.next()
"#;
    // Both return 1 (independent state)
    assert_eq!(eval(src).unwrap(), "1");
}

#[test]
fn test_array_iterator_type_name() {
    assert_eq!(eval("type([].iterator())").unwrap(), "iterator");
}

// --- Array iterator: while loop ---

#[test]
fn test_array_iterator_sum_via_while() {
    let src = r#"
let it = [1, 2, 3, 4].iterator()
let sum = 0
while (it.has_next()) {
    sum = sum + it.next()
}
sum
"#;
    assert_eq!(eval(src).unwrap(), "10");
}

// --- Dict iterator ---

#[test]
fn test_dict_iterator_has_next() {
    let src = r#"let d = {"a": 1}
d.iterator().has_next()"#;
    assert_eq!(eval(src).unwrap(), "true");
}

#[test]
fn test_dict_iterator_empty_has_next_false() {
    let src = r#"let d = {}
d.iterator().has_next()"#;
    assert_eq!(eval(src).unwrap(), "false");
}

#[test]
fn test_dict_iterator_yields_key() {
    let src = r#"
let d = {"hello": 99}
d.iterator().next()
"#;
    assert_eq!(eval(src).unwrap(), "hello");
}

#[test]
fn test_dict_iterator_counts_all_keys() {
    let src = r#"
let it = {"a": 1, "b": 2, "c": 3}.iterator()
let count = 0
while (it.has_next()) {
    it.next()
    count = count + 1
}
count
"#;
    assert_eq!(eval(src).unwrap(), "3");
}

// --- Set iterator ---

#[test]
fn test_set_iterator_has_next_non_empty() {
    assert_eq!(eval("set([1]).iterator().has_next()").unwrap(), "true");
}

#[test]
fn test_set_iterator_has_next_empty() {
    assert_eq!(eval("set([]).iterator().has_next()").unwrap(), "false");
}

#[test]
fn test_set_iterator_drains_all() {
    let src = r#"
let it = set([7, 8, 9]).iterator()
let count = 0
while (it.has_next()) {
    it.next()
    count = count + 1
}
count
"#;
    assert_eq!(eval(src).unwrap(), "3");
}

// --- for-in loops with new collection types ---

#[test]
fn test_for_in_dict_visits_all_keys() {
    let src = r#"
let d = {"a": 1, "b": 2, "c": 3}
let count = 0
for k in d {
    count = count + 1
}
count
"#;
    assert_eq!(eval(src).unwrap(), "3");
}

#[test]
fn test_for_in_set_visits_all_elements() {
    let src = r#"
let s = set([10, 20, 30])
let sum = 0
for x in s {
    sum = sum + x
}
sum
"#;
    assert_eq!(eval(src).unwrap(), "60");
}

#[test]
fn test_for_in_string_visits_chars() {
    let src = r#"
let count = 0
for c in "hello" {
    count = count + 1
}
count
"#;
    assert_eq!(eval(src).unwrap(), "5");
}

#[test]
fn test_for_in_iterator_object() {
    let src = r#"
let it = [5, 6, 7].iterator()
let sum = 0
for x in it {
    sum = sum + x
}
sum
"#;
    assert_eq!(eval(src).unwrap(), "18");
}

// --- Custom struct iterators ---

#[test]
fn test_custom_counter_iterator() {
    let src = r#"
struct CounterIter {
    current
    max

    fn has_next(self) {
        return self.current < self.max
    }

    fn next(self) {
        let val = self.current
        self.current = self.current + 1
        return val
    }
}

let it = CounterIter { current: 0, max: 4 }
let sum = 0
while (it.has_next()) {
    sum = sum + it.next()
}
sum
"#;
    // 0+1+2+3 = 6
    assert_eq!(eval(src).unwrap(), "6");
}

#[test]
fn test_custom_fibonacci_iterator() {
    let src = r#"
struct FibIter {
    a
    b
    remaining

    fn has_next(self) {
        return self.remaining > 0
    }

    fn next(self) {
        let val = self.a
        let tmp = self.a + self.b
        self.a = self.b
        self.b = tmp
        self.remaining = self.remaining - 1
        return val
    }
}

let fib = FibIter { a: 0, b: 1, remaining: 7 }
let last = 0
while (fib.has_next()) {
    last = fib.next()
}
last
"#;
    // 7th fib (0-indexed): 0,1,1,2,3,5,8 → last = 8
    assert_eq!(eval(src).unwrap(), "8");
}

#[test]
fn test_custom_range_iterator() {
    let src = r#"
struct RangeIter {
    current
    end
    step

    fn has_next(self) {
        return self.current < self.end
    }

    fn next(self) {
        let val = self.current
        self.current = self.current + self.step
        return val
    }
}

let it = RangeIter { current: 0, end: 10, step: 2 }
let count = 0
while (it.has_next()) {
    it.next()
    count = count + 1
}
count
"#;
    // 0, 2, 4, 6, 8 → 5 elements
    assert_eq!(eval(src).unwrap(), "5");
}
