use crate::modules::role::domain::RoleDomainError;

/// Validated username value object (matches the `CHECK (length(username) >= 3)`
/// constraint on the `users` table plus a stricter charset rule).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Name(String);

impl Name {
    pub fn parse(raw: &str) -> Result<Self, RoleDomainError> {
        let trimmed = raw.trim();
        if trimmed.len() < 3 || trimmed.len() > 150 {
            return Err(RoleDomainError::InvalidName);
        }
        if !trimmed
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '.')
        {
            return Err(RoleDomainError::InvalidName);
        }
        Ok(Self(trimmed.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for Name {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
