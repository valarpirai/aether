//! Standard library loader for Aether
//! Embeds stdlib .ae files in the binary and loads them at startup

/// Core utilities (range, enumerate)
pub const STDLIB_CORE: &str = include_str!("../../stdlib/core.ae");

/// Collection operations (map, filter, reduce, find, every, some)
pub const STDLIB_COLLECTIONS: &str = include_str!("../../stdlib/collections.ae");

/// Get all stdlib modules to load
pub fn stdlib_modules() -> Vec<(&'static str, &'static str)> {
    vec![
        ("core", STDLIB_CORE),
        ("collections", STDLIB_COLLECTIONS),
        // Future modules will be added here:
        // ("math", STDLIB_MATH),
        // ("string", STDLIB_STRING),
    ]
}
