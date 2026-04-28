use criterion::{black_box, criterion_group, criterion_main, Criterion};

use aether::interpreter::Evaluator;
use aether::lexer::Scanner;
use aether::parser::Parser;

fn parse_and_eval(source: &str) {
    let mut scanner = Scanner::new(source);
    let tokens = scanner.scan_tokens().unwrap();
    let mut parser = Parser::new(tokens);
    let program = parser.parse().unwrap();
    let mut eval = Evaluator::new_without_stdlib();
    for stmt in &program.statements {
        eval.exec_stmt(stmt).unwrap();
    }
}

// Arithmetic loop: tight numeric computation
const ARITH_LOOP: &str = r#"
let sum = 0
let i = 0
while (i < 10000) {
    sum = sum + i
    i = i + 1
}
"#;

// Fibonacci: recursive function calls
const FIBONACCI: &str = r#"
fn fib(n) {
    if (n <= 1) { return n }
    return fib(n - 1) + fib(n - 2)
}
let result = fib(20)
"#;

// Variable lookups: nested scopes
const SCOPE_LOOKUPS: &str = r#"
let a = 1
let b = 2
let c = 3
let d = 4
let e = 5
let i = 0
while (i < 5000) {
    let x = a + b + c + d + e
    i = i + 1
}
"#;

// String operations: concat and interpolation
const STRING_OPS: &str = r#"
let s = ""
let i = 0
while (i < 1000) {
    s = "hello" + " " + "world"
    i = i + 1
}
"#;

// Array ops: push and index
const ARRAY_OPS: &str = r#"
let arr = []
let i = 0
while (i < 1000) {
    arr.push(i)
    i = i + 1
}
let sum = 0
let j = 0
while (j < 1000) {
    sum = sum + arr[j]
    j = j + 1
}
"#;

// Function calls: many small calls
const MANY_CALLS: &str = r#"
fn add(a, b) { return a + b }
let sum = 0
let i = 0
while (i < 5000) {
    sum = add(sum, i)
    i = i + 1
}
"#;

fn bench_arithmetic_loop(c: &mut Criterion) {
    c.bench_function("arithmetic_loop_10k", |b| {
        b.iter(|| parse_and_eval(black_box(ARITH_LOOP)))
    });
}

fn bench_fibonacci(c: &mut Criterion) {
    c.bench_function("fibonacci_20", |b| {
        b.iter(|| parse_and_eval(black_box(FIBONACCI)))
    });
}

fn bench_scope_lookups(c: &mut Criterion) {
    c.bench_function("scope_lookups_5k", |b| {
        b.iter(|| parse_and_eval(black_box(SCOPE_LOOKUPS)))
    });
}

fn bench_string_ops(c: &mut Criterion) {
    c.bench_function("string_ops_1k", |b| {
        b.iter(|| parse_and_eval(black_box(STRING_OPS)))
    });
}

fn bench_array_ops(c: &mut Criterion) {
    c.bench_function("array_ops_1k", |b| {
        b.iter(|| parse_and_eval(black_box(ARRAY_OPS)))
    });
}

fn bench_many_calls(c: &mut Criterion) {
    c.bench_function("many_fn_calls_5k", |b| {
        b.iter(|| parse_and_eval(black_box(MANY_CALLS)))
    });
}

criterion_group!(
    benches,
    bench_arithmetic_loop,
    bench_fibonacci,
    bench_scope_lookups,
    bench_string_ops,
    bench_array_ops,
    bench_many_calls,
);
criterion_main!(benches);
