use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::HashSet;

use super::errors::AuthError;

// §2.2 Username: 3-20 chars, letters/numbers/underscore only, no spaces
static USERNAME_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[a-zA-Z0-9_]{3,20}$").unwrap());

// §2.2 Reserved usernames — blocked outright, plus common admin/staff patterns
static RESERVED_USERNAMES: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    [
        "admin", "administrator", "support", "system", "root", "api",
        "moderator", "owner", "chess", "official", "chessking",
    ]
    .into_iter()
    .collect()
});

// Patterns like "admin123", "support_team" etc. are also blocked (§2.2:
// "any value matching common admin/staff patterns")
static RESERVED_PATTERN_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)^(admin|administrator|support|system|root|moderator|owner|official|chessking)[_0-9]*$").unwrap()
});

/// Small embedded sample of a "top 10k leaked passwords" blocklist per
/// §2.2. In production this should be loaded from a full 10k-entry file
/// (e.g. from SecLists) at startup rather than hardcoded — this is the
/// minimal seed so the check has real teeth from day one.
static COMMON_PASSWORDS: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    [
        "password", "password1", "12345678", "123456789", "qwerty123",
        "letmein1", "welcome1", "admin123", "iloveyou1", "football1",
        "monkey123", "dragon123", "master123", "abc12345", "passw0rd",
        "sunshine1", "princess1", "starwars1", "trustno1", "superman1",
    ]
    .into_iter()
    .collect()
});

pub fn validate_username(username: &str) -> Result<(), AuthError> {
    if !USERNAME_RE.is_match(username) {
        return Err(AuthError::UsernameFormatInvalid);
    }
    let lower = username.to_lowercase();
    if RESERVED_USERNAMES.contains(lower.as_str()) || RESERVED_PATTERN_RE.is_match(username) {
        return Err(AuthError::ReservedUsername);
    }
    Ok(())
}

pub fn validate_email(email: &str) -> Result<(), AuthError> {
    // Standard RFC-ish check — deliberately not exhaustive per RFC 5322,
    // matches what real-world signup forms use.
    static EMAIL_RE: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"^[a-zA-Z0-9.!#$%&'*+/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)+$").unwrap()
    });
    if !EMAIL_RE.is_match(email) {
        return Err(AuthError::EmailFormatInvalid);
    }
    Ok(())
}

/// §2.2 Password: min 8 chars, at least one uppercase, one lowercase, one
/// number. Special character recommended but NOT required. Checked
/// against common-password blocklist.
pub fn validate_password(password: &str) -> Result<(), AuthError> {
    if password.len() < 8 {
        return Err(AuthError::PasswordTooWeak);
    }
    let has_upper = password.chars().any(|c| c.is_ascii_uppercase());
    let has_lower = password.chars().any(|c| c.is_ascii_lowercase());
    let has_digit = password.chars().any(|c| c.is_ascii_digit());
    if !(has_upper && has_lower && has_digit) {
        return Err(AuthError::PasswordTooWeak);
    }
    if COMMON_PASSWORDS.contains(password.to_lowercase().as_str()) {
        return Err(AuthError::PasswordTooWeak);
    }
    Ok(())
}
