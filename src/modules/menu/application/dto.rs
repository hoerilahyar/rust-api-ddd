use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize};
use validator::Validate;

use crate::modules::menu::domain::Menu;

/// Distinguishes "field omitted" (`None`, outer) from "field explicitly
/// `null`" (`Some(None)`) from "field set to a value" (`Some(Some(x))`).
/// Plain `Option<Option<T>>` can't do this on its own -- serde collapses a
/// JSON `null` straight to the outer `None` -- so the field is only ever
/// deserialized (via `#[serde(default, deserialize_with = ...)]`) when it's
/// actually present in the payload; when present, we always wrap in `Some`
/// and let the inner `Option<T>` handle `null` vs a real value normally.
fn de_double_option<'de, T, D>(deserializer: D) -> Result<Option<Option<T>>, D::Error>
where
    T: serde::Deserialize<'de>,
    D: Deserializer<'de>,
{
    Ok(Some(Option::<T>::deserialize(deserializer)?))
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateMenuRequest {
    pub parent_id: Option<i32>,

    #[validate(length(min = 1, max = 150, message = "name is required"))]
    pub name: String,

    #[validate(length(min = 1, max = 150, message = "slug is required"))]
    pub slug: String,

    pub path: Option<String>,
    pub icon: Option<String>,

    #[serde(default)]
    pub order_index: i32,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateMenuRequest {
    /// Omitted = don't touch. `null` = move to top-level. An id = reparent
    /// under that menu (validated against cycles in the service layer).
    #[serde(default, deserialize_with = "de_double_option")]
    pub parent_id: Option<Option<i32>>,

    #[validate(length(min = 1, max = 150, message = "name is required"))]
    pub name: Option<String>,

    pub path: Option<String>,

    #[serde(default, deserialize_with = "de_double_option")]
    pub icon: Option<Option<String>>,
    pub order_index: Option<i32>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct AssignMenuPermissionRequest {
    #[validate(length(min = 1, message = "permission name is required"))]
    pub permission: String,
}

#[derive(Debug, Serialize)]
pub struct MenuResponse {
    pub id: i32,
    pub parent_id: Option<i32>,
    pub name: String,
    pub slug: String,
    pub path: Option<String>,
    pub icon: Option<String>,
    pub order_index: i32,
    pub is_active: bool,
    pub permissions: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Menu> for MenuResponse {
    fn from(m: Menu) -> Self {
        Self {
            id: m.id,
            parent_id: m.parent_id,
            name: m.name,
            slug: m.slug,
            path: m.path,
            icon: m.icon,
            order_index: m.order_index,
            is_active: m.is_active,
            permissions: m.permissions,
            created_at: m.created_at,
            updated_at: m.updated_at,
        }
    }
}

/// Nested tree node returned by `GET /menus/tree` (admin, unfiltered) and
/// `GET /me/menu` (filtered to the caller's own permissions).
#[derive(Debug, Clone, Serialize)]
pub struct MenuTreeNode {
    pub id: i32,
    pub parent_id: Option<i32>,
    pub name: String,
    pub slug: String,
    pub path: Option<String>,
    pub icon: Option<String>,
    pub order_index: i32,
    pub is_active: bool,
    pub permissions: Vec<String>,
    pub children: Vec<MenuTreeNode>,
}

impl From<Menu> for MenuTreeNode {
    fn from(m: Menu) -> Self {
        Self {
            id: m.id,
            parent_id: m.parent_id,
            name: m.name,
            slug: m.slug,
            path: m.path,
            icon: m.icon,
            order_index: m.order_index,
            is_active: m.is_active,
            permissions: m.permissions,
            children: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The whole point of `de_double_option`: three distinct wire states
    /// must map to three distinct Rust values. A regular `Option<Option<i32>>`
    /// without the custom deserializer would collapse "omitted" and
    /// "explicit null" into the same `None`, silently turning every
    /// "move to top-level" request into a no-op.
    #[test]
    fn omitted_parent_id_means_dont_touch() {
        let req: UpdateMenuRequest = serde_json::from_str(r#"{"name": "Reports"}"#).unwrap();
        assert_eq!(req.parent_id, None);
    }

    #[test]
    fn explicit_null_parent_id_means_move_to_top_level() {
        let req: UpdateMenuRequest =
            serde_json::from_str(r#"{"name": "Reports", "parent_id": null}"#).unwrap();
        assert_eq!(req.parent_id, Some(None));
    }

    #[test]
    fn explicit_value_parent_id_means_reparent() {
        let req: UpdateMenuRequest =
            serde_json::from_str(r#"{"name": "Reports", "parent_id": 7}"#).unwrap();
        assert_eq!(req.parent_id, Some(Some(7)));
    }

    #[test]
    fn other_optional_fields_still_omit_normally() {
        let req: UpdateMenuRequest = serde_json::from_str(r#"{}"#).unwrap();
        assert_eq!(req.name, None);
        assert_eq!(req.is_active, None);
        assert_eq!(req.parent_id, None);
    }
}
