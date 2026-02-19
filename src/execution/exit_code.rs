/// Clamps exit codes to POSIX-compliant range (0-255).
///
/// POSIX standards require exit codes to be in the range 0-255.
/// This function ensures compliance by:
/// - Mapping negative codes to 1 (generic failure)
/// - Capping codes > 255 to 255 (maximum valid exit code)
/// - Passing through valid codes (0-255) unchanged
///
/// # Arguments
/// * `code` - The raw exit code to clamp
///
/// # Returns
/// An exit code in the range 0-255
///
/// # Examples
/// ```
/// use syntropy::execution::clamp_exit_code;
///
/// assert_eq!(clamp_exit_code(-1), 1);    // Negative → 1
/// assert_eq!(clamp_exit_code(0), 0);      // Valid → unchanged
/// assert_eq!(clamp_exit_code(127), 127);  // Valid → unchanged
/// assert_eq!(clamp_exit_code(300), 255);  // >255 → 255
/// ```
pub fn clamp_exit_code(code: i32) -> i32 {
    match code {
        code if code < 0 => 1,
        code if code > 255 => 255,
        code => code,
    }
}
