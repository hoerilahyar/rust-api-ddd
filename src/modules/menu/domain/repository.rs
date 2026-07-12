use async_trait::async_trait;

use crate::modules::menu::domain::Menu;
use crate::shared::errors::AppError;

#[async_trait]
pub trait MenuRepository: Send + Sync {
    async fn find_by_id(&self, id: i32) -> Result<Option<Menu>, AppError>;
    async fn find_by_slug(&self, slug: &str) -> Result<Option<Menu>, AppError>;

    /// Every non-deleted menu (active + inactive), flat. Used to build both
    /// the admin tree and the "visible to me" tree, and for cycle detection
    /// when reparenting -- menus are a small, bounded dataset so fetching
    /// everything and working with it in memory is simpler than N queries.
    async fn list_all(&self) -> Result<Vec<Menu>, AppError>;

    async fn create(
        &self,
        parent_id: Option<i32>,
        name: &str,
        slug: &str,
        path: Option<&str>,
        icon: Option<&str>,
        order_index: i32,
    ) -> Result<Menu, AppError>;

    /// `parent_id: None` = don't touch; `Some(None)` = move to top-level;
    /// `Some(Some(id))` = reparent under `id`.
    #[allow(clippy::too_many_arguments)]
    async fn update(
        &self,
        id: i32,
        parent_id: Option<Option<i32>>,
        name: Option<&str>,
        path: Option<&str>,
        icon: Option<&str>,
        order_index: Option<i32>,
        is_active: Option<bool>,
    ) -> Result<Menu, AppError>;

    /// Soft delete (sets `deleted_at`). Cascades to all descendants in the
    /// tree, not just the row itself -- the DB's `ON DELETE CASCADE` on
    /// `parent_id` only fires on a hard delete, so the postgres
    /// implementation walks the subtree explicitly (recursive CTE) to avoid
    /// leaving orphaned children pointing at a deleted parent.
    async fn delete(&self, id: i32) -> Result<(), AppError>;

    async fn assign_permission(&self, menu_id: i32, permission_id: i32) -> Result<(), AppError>;
    async fn revoke_permission(&self, menu_id: i32, permission_id: i32) -> Result<(), AppError>;
    async fn find_permission_by_name(&self, name: &str) -> Result<Option<(i32, String)>, AppError>;
}
