//! Regression tests for clippy fixes
//! These tests ensure refactored code (renamed methods, suppressions) still works correctly.
//! Focus on USE CASES, not just code coverage.

use aether::interpreter::Evaluator;
use aether::lexer::Scanner;
use aether::parser::Parser;

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
        if let aether::parser::ast::Stmt::Expr(expr) = last {
            let value = evaluator.eval_expr(expr).map_err(|e| e.to_string())?;
            return Ok(format!("{}", value));
        }
        evaluator.exec_stmt(last).map_err(|e| e.to_string())?;
    }

    Ok("null".to_string())
}

// ============================================================================
// USE CASE: Set operations with various value types
// ============================================================================

#[test]
fn use_case_set_with_integers() {
    let source = r#"
let nums = set([1, 2, 3, 2, 1])
nums.size
"#;
    let result = eval(source);
    assert_eq!(result.unwrap(), "3", "Set should deduplicate integers");
}

#[test]
fn use_case_set_with_strings() {
    let source = r#"
let words = set(["hello", "world", "hello"])
words.size
"#;
    let result = eval(source);
    assert_eq!(result.unwrap(), "2", "Set should deduplicate strings");
}

#[test]
fn use_case_set_with_mixed_hashable_types() {
    let source = r#"
let mixed = set([1, "hello", true, 1, "hello", false, true])
mixed.size
"#;
    let result = eval(source);
    assert_eq!(
        result.unwrap(),
        "4",
        "Set should handle mixed hashable types"
    );
}

#[test]
fn use_case_set_union_workflow() {
    let source = r#"
let admins = set(["alice", "bob"])
let moderators = set(["bob", "charlie"])
let all_staff = admins.union(moderators)
all_staff.size
"#;
    let result = eval(source);
    assert_eq!(result.unwrap(), "3", "Union should combine unique members");
}

#[test]
fn use_case_set_intersection_workflow() {
    let source = r#"
let online = set([1, 2, 3, 4])
let premium = set([3, 4, 5, 6])
let online_premium = online.intersection(premium)
online_premium.size
"#;
    let result = eval(source);
    assert_eq!(
        result.unwrap(),
        "2",
        "Intersection should find common elements"
    );
}

#[test]
fn use_case_set_difference_workflow() {
    let source = r#"
let all_users = set([1, 2, 3, 4, 5])
let banned_users = set([3, 5])
let active_users = all_users.difference(banned_users)
active_users.size
"#;
    let result = eval(source);
    assert_eq!(
        result.unwrap(),
        "3",
        "Difference should remove banned users"
    );
}

#[test]
fn use_case_set_membership_check() {
    let source = r#"
let allowed_roles = set(["admin", "editor", "viewer"])
let user_role = "editor"
allowed_roles.contains(user_role)
"#;
    let result = eval(source);
    assert_eq!(result.unwrap(), "true", "Should verify role membership");
}

#[test]
fn use_case_set_add_remove_workflow() {
    let source = r#"
let tags = set(["urgent", "bug"])
tags.add("security")
tags.remove("bug")
tags.size
"#;
    let result = eval(source);
    assert_eq!(
        result.unwrap(),
        "2",
        "Add/remove should modify set correctly"
    );
}

#[test]
fn use_case_set_to_array_for_iteration() {
    let source = r#"
let unique_ids = set([101, 102, 103])
let id_array = unique_ids.to_array()
len(id_array)
"#;
    let result = eval(source);
    assert_eq!(
        result.unwrap(),
        "3",
        "to_array() should convert for iteration"
    );
}

#[test]
fn use_case_set_subset_check() {
    let source = r#"
let required_permissions = set(["read", "write"])
let user_permissions = set(["read", "write", "delete"])
required_permissions.is_subset(user_permissions)
"#;
    let result = eval(source);
    assert_eq!(
        result.unwrap(),
        "true",
        "Should verify user has all required permissions"
    );
}

// ============================================================================
// USE CASE: Module imports (testing renamed methods)
// ============================================================================

#[test]
fn use_case_import_utility_functions() {
    // Module should provide reusable utilities
    let result = std::env::set_current_dir("tests/test_modules");
    assert!(result.is_ok(), "Should be able to change to test directory");

    let source = r#"
from math_utils import double, triple
let x = 10
double(x) + triple(x)
"#;
    let result = eval(source);
    std::env::set_current_dir("../..").ok();

    assert_eq!(
        result.unwrap(),
        "50",
        "Should import and use multiple functions"
    );
}

#[test]
fn use_case_import_with_alias_for_clarity() {
    let result = std::env::set_current_dir("tests/test_modules");
    assert!(result.is_ok());

    let source = r#"
from math_utils import double as twice
let val = 7
twice(val)
"#;
    let result = eval(source);
    std::env::set_current_dir("../..").ok();

    assert_eq!(
        result.unwrap(),
        "14",
        "Should use aliased import for clarity"
    );
}

#[test]
fn use_case_namespace_import_for_organization() {
    let result = std::env::set_current_dir("tests/test_modules");
    assert!(result.is_ok());

    let source = r#"
import math_utils
let base = 4
math_utils.square(base)
"#;
    let result = eval(source);
    std::env::set_current_dir("../..").ok();

    assert_eq!(
        result.unwrap(),
        "16",
        "Namespace import should organize code"
    );
}

// ============================================================================
// USE CASE: Edge cases and error handling
// ============================================================================

#[test]
fn use_case_empty_set_operations() {
    let source = r#"
let empty1 = set([])
let empty2 = set([])
let union = empty1.union(empty2)
union.size
"#;
    let result = eval(source);
    assert_eq!(result.unwrap(), "0", "Empty sets should work in operations");
}

#[test]
fn use_case_set_with_null_values() {
    let source = r#"
let values = set([null, 1, null, 2])
values.size
"#;
    let result = eval(source);
    assert_eq!(result.unwrap(), "3", "null should be deduplicated in sets");
}

#[test]
fn use_case_set_error_non_hashable() {
    let source = r#"
set([[1, 2], [3, 4]])
"#;
    let result = eval(source);
    assert!(result.is_err(), "Should reject non-hashable values");
    assert!(
        result.unwrap_err().contains("hashable"),
        "Error should mention hashability"
    );
}

// ============================================================================
// USE CASE: Real-world scenarios
// ============================================================================

#[test]
fn use_case_deduplication_workflow() {
    let source = r#"
let log_entries = [101, 102, 101, 103, 102, 104]
let unique_ids = set(log_entries)
unique_ids.size
"#;
    let result = eval(source);
    assert_eq!(result.unwrap(), "4", "Should deduplicate log IDs");
}

#[test]
fn use_case_access_control_check() {
    let source = r#"
let user_roles = set(["user", "contributor"])
let admin_roles = set(["admin", "moderator"])
let has_admin = user_roles.intersection(admin_roles)
has_admin.size == 0
"#;
    let result = eval(source);
    assert_eq!(result.unwrap(), "true", "Should detect non-admin user");
}

#[test]
fn use_case_feature_flag_check() {
    let source = r#"
let enabled_features = set(["dark_mode", "notifications", "analytics"])
enabled_features.contains("dark_mode") && enabled_features.contains("notifications")
"#;
    let result = eval(source);
    assert_eq!(
        result.unwrap(),
        "true",
        "Should check multiple feature flags"
    );
}

#[test]
fn use_case_tag_management() {
    let source = r#"
let post_tags = set(["rust", "programming"])
post_tags.add("tutorial")
post_tags.add("beginner")
let tutorial_tags = set(["tutorial", "guide"])
post_tags.intersection(tutorial_tags).size > 0
"#;
    let result = eval(source);
    assert_eq!(result.unwrap(), "true", "Should find common tags");
}
