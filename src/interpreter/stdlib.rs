//! Standard library loader for Aether
//! Embeds stdlib .ae files in the binary and loads them at startup

/// Core utilities (range, enumerate)
pub const STDLIB_CORE: &str = include_str!("../../stdlib/core.ae");

/// Get all stdlib modules to load
pub fn stdlib_modules() -> Vec<(&'static str, &'static str)> {
    vec![
        ("core", STDLIB_CORE),
        // Future modules will be added here:
        // ("collections", STDLIB_COLLECTIONS),
        // ("math", STDLIB_MATH),
        // ("string", STDLIB_STRING),
    ]
}
