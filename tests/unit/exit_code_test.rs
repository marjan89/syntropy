//! Unit tests for exit code clamping functionality
//!
//! Tests the clamp_exit_code function that ensures POSIX compliance
//! by clamping exit codes to the valid 0-255 range.

use syntropy::execution::clamp_exit_code;

// ============================================================================
// Negative Exit Code Clamping Tests
// ============================================================================

#[test]
fn test_clamp_negative_one() {
    assert_eq!(clamp_exit_code(-1), 1);
}

#[test]
fn test_clamp_negative_large() {
    assert_eq!(clamp_exit_code(-127), 1);
}

#[test]
fn test_clamp_negative_min() {
    assert_eq!(clamp_exit_code(i32::MIN), 1);
}

// ============================================================================
// Large Exit Code Clamping Tests
// ============================================================================

#[test]
fn test_clamp_just_over_max() {
    assert_eq!(clamp_exit_code(256), 255);
}

#[test]
fn test_clamp_moderately_large() {
    assert_eq!(clamp_exit_code(300), 255);
}

#[test]
fn test_clamp_very_large() {
    assert_eq!(clamp_exit_code(i32::MAX), 255);
}

// ============================================================================
// Valid Exit Code Pass-Through Tests
// ============================================================================

#[test]
fn test_pass_through_zero() {
    assert_eq!(clamp_exit_code(0), 0);
}

#[test]
fn test_pass_through_one() {
    assert_eq!(clamp_exit_code(1), 1);
}

#[test]
fn test_pass_through_command_not_found() {
    assert_eq!(clamp_exit_code(127), 127);
}

#[test]
fn test_pass_through_max_valid() {
    assert_eq!(clamp_exit_code(255), 255);
}

#[test]
fn test_pass_through_middle_range() {
    assert_eq!(clamp_exit_code(42), 42);
    assert_eq!(clamp_exit_code(100), 100);
    assert_eq!(clamp_exit_code(200), 200);
}
