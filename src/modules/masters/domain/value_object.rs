use crate::modules::masters::domain::errors::{MasterGroupDomainError, MasterItemDomainError};

/// Display-name value object for master groups/items (e.g. "Payment Method").
/// Allows letters, numbers, spaces, and a few common punctuation marks —
/// unlike Role::Name, this is NOT a slug/identifier, it's a human label.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Name(String);

impl Name {
    pub fn parse(raw: &str) -> Result<Self, MasterGroupDomainError> {
        let trimmed = raw.trim();
        if trimmed.is_empty() || trimmed.len() > 150 {
            return Err(MasterGroupDomainError::InvalidName);
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

/// Same validation, item-scoped error type — kept separate so item call
/// sites get an item-specific error variant instead of a group one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemName(String);

impl ItemName {
    pub fn parse(raw: &str) -> Result<Self, MasterItemDomainError> {
        let trimmed = raw.trim();
        if trimmed.is_empty() || trimmed.len() > 150 {
            return Err(MasterItemDomainError::InvalidName);
        }
        Ok(Self(trimmed.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}
