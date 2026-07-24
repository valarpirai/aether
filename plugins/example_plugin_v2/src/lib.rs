//! Example Aether plugin using V2 protocol (String, Vec, HashMap support)

use aether_plugin::*;
use std::collections::HashMap;

/// Greet someone by name
#[aether_export]
fn greet(name: String) -> String {
    format!("Hello, {}!", name)
}

/// Convert string to uppercase
#[aether_export]
fn to_upper(s: String) -> String {
    s.to_uppercase()
}

/// Repeat a string n times
#[aether_export]
fn repeat_string(s: String, n: i64) -> String {
    s.repeat(n as usize)
}

/// Sort an array of integers
#[aether_export]
fn sort_array(mut nums: Vec<i64>) -> Vec<i64> {
    nums.sort();
    nums
}

/// Sum an array of integers
#[aether_export]
fn sum_array(nums: Vec<i64>) -> i64 {
    nums.iter().sum()
}

/// Reverse an array
#[aether_export]
fn reverse_array(mut nums: Vec<i64>) -> Vec<i64> {
    nums.reverse();
    nums
}

/// Sum the values of a dict
#[aether_export]
fn sum_values(scores: HashMap<String, i64>) -> i64 {
    scores.values().sum()
}

/// Increment every value in a dict by one, returning a new dict
#[aether_export]
fn increment_values(scores: HashMap<String, i64>) -> HashMap<String, i64> {
    scores.into_iter().map(|(k, v)| (k, v + 1)).collect()
}

/// Divide two integers. Returns Err on divide-by-zero so the failure reaches
/// Aether as a catchable error rather than a silent null.
#[aether_export]
fn checked_div(a: i64, b: i64) -> Result<i64, String> {
    if b == 0 {
        Err("division by zero".to_string())
    } else {
        Ok(a / b)
    }
}

// Register all V2 functions (complex types: String, Vec, HashMap)
aether_plugin_init_v2!(
    greet,
    to_upper,
    repeat_string,
    sort_array,
    sum_array,
    reverse_array,
    sum_values,
    increment_values,
    checked_div
);
