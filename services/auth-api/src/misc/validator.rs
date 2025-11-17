use regex::Regex;

use crate::http::AppError;

/// Validates email format using regex
/// Matches standard email format: user@domain.com
pub fn validate_email(email: &str) -> Result<bool, AppError> {
    // RFC 5322 compliant email regex (simplified but robust)
    let email_regex = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
    email_regex
        .is_match(email)
        .then_some(true)
        .ok_or_else(|| AppError::Validation("Invalid email format".to_string()))
}

/// Validates password strength using regex
/// Requirements:
/// - At least 8 characters long
/// - At least one uppercase letter
/// - At least one lowercase letter  
/// - At least one digit
/// - At least one special character
pub fn validate_password(password: &str) -> Result<bool, AppError> {
    // Check minimum length
    if password.len() < 8 {
        return Err(AppError::Validation("Password must be at least 8 characters long".to_string()));
    }

    // Check for at least one uppercase letter
    let has_uppercase = Regex::new(r"[A-Z]").unwrap().is_match(password);

    // Check for at least one lowercase letter
    let has_lowercase = Regex::new(r"[a-z]").unwrap().is_match(password);

    // Check for at least one digit
    let has_digit = Regex::new(r"[0-9]").unwrap().is_match(password);

    // Check for at least one special character - using a different approach
    // Match any non-alphanumeric character
    let has_special = !password.chars().all(|c| c.is_alphanumeric());

    (has_uppercase && has_lowercase && has_digit && has_special)
        .then_some(true)
        .ok_or_else(|| AppError::Validation("Password must contain uppercase, lowercase, digit, and special character".to_string()))
}
