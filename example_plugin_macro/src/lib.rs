//! Example Aether plugin using the #[aether_export] macro
//!
//! This demonstrates how simple plugin authoring becomes with the macro.
//! Compare this to example_plugin/src/lib.rs to see the difference!

use aether_plugin::*;

/// Add two numbers
#[aether_export]
fn add(a: i64, b: i64) -> i64 {
    a + b
}

/// Multiply two numbers
#[aether_export]
fn multiply(a: i64, b: i64) -> i64 {
    a * b
}

/// Raise base to power
#[aether_export]
fn power(base: i64, exp: i64) -> i64 {
    if exp < 0 {
        return 0; // Can't do negative exponents with integers
    }
    (base as f64).powi(exp as i32) as i64
}

/// Check if number is even
#[aether_export]
fn is_even(n: i64) -> i64 {
    if n % 2 == 0 {
        1
    } else {
        0
    }
}

/// Compute factorial
#[aether_export]
fn factorial(n: i64) -> i64 {
    if n < 0 {
        return 0; // Invalid input
    }
    if n == 0 || n == 1 {
        return 1;
    }
    let mut result = 1i64;
    for i in 2..=n {
        result = result.saturating_mul(i);
    }
    result
}

/// Compute greatest common divisor
#[aether_export]
fn gcd(a: i64, b: i64) -> i64 {
    let mut a = a.abs();
    let mut b = b.abs();
    while b != 0 {
        let temp = b;
        b = a % b;
        a = temp;
    }
    a
}

// Auto-generate the plugin initialization code
aether_plugin_init!(add, multiply, power, is_even, factorial, gcd);
