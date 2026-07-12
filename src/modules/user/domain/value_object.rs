use crate::modules::user::domain::errors::UserDomainError;

/// Validated email value object. Keeps "what is a valid email" in the
/// domain layer instead of scattered across DTOs/handlers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Email(String);

impl Email {
    pub fn parse(raw: &str) -> Result<Self, UserDomainError> {
        let trimmed = raw.trim();
        if trimmed.len() < 3 || !trimmed.contains('@') || !trimmed.contains('.') {
            return Err(UserDomainError::InvalidEmail);
        }
        Ok(Self(trimmed.to_lowercase()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for Email {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Validated username value object (matches the `CHECK (length(username) >= 3)`
/// constraint on the `users` table plus a stricter charset rule).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Username(String);

impl Username {
    pub fn parse(raw: &str) -> Result<Self, UserDomainError> {
        let trimmed = raw.trim();
        if trimmed.len() < 3 || trimmed.len() > 150 {
            return Err(UserDomainError::InvalidUsername);
        }
        if !trimmed
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '.')
        {
            return Err(UserDomainError::InvalidUsername);
        }
        Ok(Self(trimmed.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for Username {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
