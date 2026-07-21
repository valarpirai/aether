//! Example Aether plugin using V2 protocol (String, Vec support)

use aether_plugin::*;

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

// Register all V2 functions (complex types: String, Vec)
aether_plugin_init_v2!(
    greet,
    to_upper,
    repeat_string,
    sort_array,
    sum_array,
    reverse_array
);
