use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;

use chrono::Utc;

use crate::modules::audit_trail_log::domain::entity::AuditTrailLog;
use crate::modules::menu::application::dto::{CreateMenuRequest, MenuTreeNode, UpdateMenuRequest};
use crate::modules::menu::application::service::MenuService;
use crate::modules::menu::domain::{Menu, MenuDomainError, MenuRepository};
use crate::shared::cache::{CacheRepository, RedisCacheRepository};
use crate::shared::context::current_request_context;
use crate::shared::contracts::AuditTrailRecorder;
use crate::shared::domain::PaginationParams;
use crate::shared::errors::AppError;

const ENTITY_TYPE: &str = "menu";

/// Fires an audit trail write in the background so a slow/unavailable audit
/// sink never blocks or fails the actual menu mutation. Errors are logged,
/// not propagated -- consistent with how `activity_recorder` calls are
/// fire-and-forget elsewhere in the codebase.
fn spawn_audit_log(
    audit: Arc<dyn AuditTrailRecorder>,
    actor_id: i32,
    action: &'static str,
    menu_id: i32,
    old_values: Option<&Menu>,
    new_values: Option<&Menu>,
) {
    let ctx = current_request_context();
    let log = AuditTrailLog {
        id: 0,
        user_id: Some(actor_id),
        action: action.to_string(),
        entity_type: ENTITY_TYPE.to_string(),
        entity_id: Some(menu_id.to_string()),
        old_values: old_values.and_then(|m| serde_json::to_value(m).ok()),
        new_values: new_values.and_then(|m| serde_json::to_value(m).ok()),
        ip_address: ctx.ip_address,
        user_agent: ctx.user_agent,
        description: Some(format!("menu id {menu_id}")),
        created_at: Utc::now(),
    };

    tokio::spawn(async move {
        if let Err(err) = audit.record_audit_trail_log(log).await {
            tracing::error!(error = ?err, menu_id, action, "failed to record menu audit trail log");
        }
    });
}

const CACHE_TTL: Duration = Duration::from_secs(300);

fn cache_key(id: i32) -> String {
    format!("menu:id:{id}")
}

pub struct MenuServiceImpl {
    audit: Arc<dyn AuditTrailRecorder>,
    repo: Arc<dyn MenuRepository>,
    cache: Arc<RedisCacheRepository>,
}

impl MenuServiceImpl {
    pub fn new(
        audit: Arc<dyn AuditTrailRecorder>,
        repo: Arc<dyn MenuRepository>,
        cache: Arc<RedisCacheRepository>,
    ) -> Self {
        Self { audit, repo, cache }
    }
}

/// True if reparenting `id` under `new_parent_id` would make `id` its own
/// ancestor -- either directly (`new_parent_id == id`) or by walking up
/// `new_parent_id`'s own parent chain and finding `id` somewhere in it.
fn would_create_cycle(id: i32, new_parent_id: i32, all: &[Menu]) -> bool {
    if new_parent_id == id {
        return true;
    }

    let parent_of: HashMap<i32, Option<i32>> = all.iter().map(|m| (m.id, m.parent_id)).collect();
    let mut current = Some(new_parent_id);
    while let Some(cur) = current {
        if cur == id {
            return true;
        }
        current = parent_of.get(&cur).copied().flatten();
    }
    false
}

/// Nests a flat list into a tree via `parent_id`, sorted by `order_index`.
/// A menu whose parent isn't present in `menus` (already filtered out, e.g.
/// by `filter_visible`) is dropped along with the rest of its subtree
/// instead of being promoted to root -- an invisible/deleted parent hides
/// its children too.
fn build_tree(menus: Vec<Menu>) -> Vec<MenuTreeNode> {
    let mut children: HashMap<i32, Vec<Menu>> = HashMap::new();
    let mut roots: Vec<Menu> = Vec::new();

    for m in menus {
        match m.parent_id {
            Some(pid) => children.entry(pid).or_default().push(m),
            None => roots.push(m),
        }
    }

    fn attach(menu: Menu, children: &mut HashMap<i32, Vec<Menu>>) -> MenuTreeNode {
        let mut node = MenuTreeNode::from(menu);
        if let Some(mut kids) = children.remove(&node.id) {
            kids.sort_by_key(|k| k.order_index);
            node.children = kids.into_iter().map(|k| attach(k, children)).collect();
        }
        node
    }

    roots.sort_by_key(|r| r.order_index);
    roots
        .into_iter()
        .map(|r| attach(r, &mut children))
        .collect()
}

/// Keeps only active menus the caller can see: either the menu has no
/// permission requirement (visible to any authenticated user), or the
/// caller holds at least one of the permissions mapped to it.
fn filter_visible(menus: Vec<Menu>, user_permissions: &[String]) -> Vec<Menu> {
    menus
        .into_iter()
        .filter(|m| {
            m.is_active
                && (m.permissions.is_empty()
                    || m.permissions.iter().any(|p| user_permissions.contains(p)))
        })
        .collect()
}

#[async_trait]
impl MenuService for MenuServiceImpl {
    async fn get_by_id(&self, id: i32) -> Result<Menu, AppError> {
        let key = cache_key(id);
        if let Some(cached) = self.cache.get::<Menu>(&key).await? {
            return Ok(cached);
        }

        let menu = self
            .repo
            .find_by_id(id)
            .await?
            .ok_or(MenuDomainError::NotFound)?;

        self.cache.set(&key, &menu, CACHE_TTL).await?;
        Ok(menu)
    }

    async fn list(&self, pagination: &PaginationParams) -> Result<(Vec<Menu>, i64), AppError> {
        // Menus are a small, bounded dataset (a handful to a few hundred
        // rows) -- paginating in memory here avoids a second, more complex
        // SQL query just for the admin table view.
        let mut all = self.repo.list_all().await?;

        if let Some(search) = pagination.search.as_deref().filter(|s| !s.is_empty()) {
            let needle = search.to_lowercase();
            all.retain(|m| {
                m.name.to_lowercase().contains(&needle) || m.slug.to_lowercase().contains(&needle)
            });
        }

        let total = all.len() as i64;
        let (page, limit) = pagination.normalized();
        let offset = ((page - 1) * limit).max(0) as usize;
        let items = all.into_iter().skip(offset).take(limit as usize).collect();
        Ok((items, total))
    }

    async fn tree(&self) -> Result<Vec<MenuTreeNode>, AppError> {
        let all = self.repo.list_all().await?;
        Ok(build_tree(all))
    }

    async fn visible_tree(
        &self,
        user_permissions: &[String],
    ) -> Result<Vec<MenuTreeNode>, AppError> {
        let all = self.repo.list_all().await?;
        Ok(build_tree(filter_visible(all, user_permissions)))
    }

    async fn create(&self, req: CreateMenuRequest, actor_id: i32) -> Result<Menu, AppError> {
        if let Some(parent_id) = req.parent_id {
            self.repo
                .find_by_id(parent_id)
                .await?
                .ok_or(MenuDomainError::ParentNotFound)?;
        }

        if self.repo.find_by_slug(&req.slug).await?.is_some() {
            return Err(MenuDomainError::SlugTaken.into());
        }

        let menu = self
            .repo
            .create(
                req.parent_id,
                &req.name,
                &req.slug,
                req.path.as_deref(),
                req.icon.as_deref(),
                req.order_index,
            )
            .await?;

        spawn_audit_log(
            self.audit.clone(),
            actor_id,
            "menu.create",
            menu.id,
            None,
            Some(&menu),
        );

        Ok(menu)
    }

    async fn update(
        &self,
        id: i32,
        req: UpdateMenuRequest,
        actor_id: i32,
    ) -> Result<Menu, AppError> {
        if let Some(Some(new_parent)) = req.parent_id {
            let all = self.repo.list_all().await?;
            if !all.iter().any(|m| m.id == new_parent) {
                return Err(MenuDomainError::ParentNotFound.into());
            }
            if would_create_cycle(id, new_parent, &all) {
                return Err(MenuDomainError::CircularParent.into());
            }
        }

        let existing = self
            .repo
            .find_by_id(id)
            .await?
            .ok_or(MenuDomainError::NotFound)?;

        let menu = self
            .repo
            .update(
                id,
                req.parent_id,
                req.name.as_deref(),
                req.path.as_deref(),
                req.icon.as_deref(),
                req.order_index,
                req.is_active,
            )
            .await?;

        spawn_audit_log(
            self.audit.clone(),
            actor_id,
            "menu.update",
            id,
            Some(&existing),
            Some(&menu),
        );

        self.cache.delete(&cache_key(id)).await?;
        Ok(menu)
    }

    async fn delete(&self, id: i32, actor_id: i32) -> Result<(), AppError> {
        let existing = self
            .repo
            .find_by_id(id)
            .await?
            .ok_or(MenuDomainError::NotFound)?;

        self.repo.delete(id).await?;

        spawn_audit_log(
            self.audit.clone(),
            actor_id,
            "menu.delete",
            id,
            Some(&existing),
            None,
        );

        self.cache.delete(&cache_key(id)).await?;
        Ok(())
    }

    async fn assign_permission(&self, menu_id: i32, permission_name: &str) -> Result<(), AppError> {
        let (permission_id, _) = self
            .repo
            .find_permission_by_name(permission_name)
            .await?
            .ok_or_else(|| {
                AppError::NotFound(format!("permission '{permission_name}' not found"))
            })?;

        self.repo.assign_permission(menu_id, permission_id).await?;
        self.cache.delete(&cache_key(menu_id)).await?;
        Ok(())
    }

    async fn revoke_permission(&self, menu_id: i32, permission_name: &str) -> Result<(), AppError> {
        let (permission_id, _) = self
            .repo
            .find_permission_by_name(permission_name)
            .await?
            .ok_or_else(|| {
                AppError::NotFound(format!("permission '{permission_name}' not found"))
            })?;

        self.repo.revoke_permission(menu_id, permission_id).await?;
        self.cache.delete(&cache_key(menu_id)).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    /// Builds a bare-bones `Menu` for tests -- only `id`/`parent_id` matter
    /// for tree/cycle tests, the rest are filled with harmless defaults.
    fn menu(id: i32, parent_id: Option<i32>, order_index: i32) -> Menu {
        Menu {
            id,
            parent_id,
            name: format!("menu-{id}"),
            slug: format!("menu-{id}"),
            path: None,
            icon: None,
            order_index,
            is_active: true,
            permissions: Vec::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn menu_with_permissions(id: i32, parent_id: Option<i32>, permissions: &[&str]) -> Menu {
        let mut m = menu(id, parent_id, 0);
        m.permissions = permissions.iter().map(|s| s.to_string()).collect();
        m
    }

    // ---- build_tree ----

    #[test]
    fn build_tree_nests_children_under_their_parent() {
        let menus = vec![menu(1, None, 0), menu(2, Some(1), 0), menu(3, Some(1), 0)];

        let tree = build_tree(menus);

        assert_eq!(tree.len(), 1);
        assert_eq!(tree[0].id, 1);
        assert_eq!(tree[0].children.len(), 2);
        assert_eq!(tree[0].children[0].id, 2);
        assert_eq!(tree[0].children[1].id, 3);
    }

    #[test]
    fn build_tree_sorts_siblings_by_order_index() {
        let menus = vec![
            menu(1, None, 0),
            menu(2, Some(1), 5),
            menu(3, Some(1), 1),
            menu(4, Some(1), 3),
        ];

        let tree = build_tree(menus);

        let child_ids: Vec<i32> = tree[0].children.iter().map(|c| c.id).collect();
        assert_eq!(child_ids, vec![3, 4, 2]);
    }

    #[test]
    fn build_tree_nests_multiple_levels_deep() {
        let menus = vec![menu(1, None, 0), menu(2, Some(1), 0), menu(3, Some(2), 0)];

        let tree = build_tree(menus);

        assert_eq!(tree[0].children[0].id, 2);
        assert_eq!(tree[0].children[0].children[0].id, 3);
    }

    #[test]
    fn build_tree_drops_subtree_when_parent_is_missing_from_input() {
        // Simulates an invisible/filtered-out parent: children pointing at
        // an id that isn't in the input list must not become roots, and
        // must not surface anywhere in the resulting tree.
        let menus = vec![menu(1, None, 0), menu(2, Some(99), 0), menu(3, Some(2), 0)];

        let tree = build_tree(menus);

        assert_eq!(tree.len(), 1);
        assert_eq!(tree[0].id, 1);
        assert!(tree[0].children.is_empty());
    }

    // ---- would_create_cycle ----

    #[test]
    fn cycle_detected_when_reparenting_under_self() {
        let all = vec![menu(1, None, 0)];
        assert!(would_create_cycle(1, 1, &all));
    }

    #[test]
    fn cycle_detected_when_reparenting_under_own_descendant() {
        // 1 -> 2 -> 3; trying to move 1 under 3 (its own grandchild).
        let all = vec![menu(1, None, 0), menu(2, Some(1), 0), menu(3, Some(2), 0)];
        assert!(would_create_cycle(1, 3, &all));
    }

    #[test]
    fn no_cycle_when_reparenting_under_unrelated_menu() {
        let all = vec![menu(1, None, 0), menu(2, None, 0), menu(3, Some(1), 0)];
        assert!(!would_create_cycle(3, 2, &all));
    }

    #[test]
    fn no_cycle_when_reparenting_under_a_sibling() {
        let all = vec![menu(1, None, 0), menu(2, Some(1), 0), menu(3, Some(1), 0)];
        assert!(!would_create_cycle(2, 3, &all));
    }

    // ---- filter_visible ----

    #[test]
    fn filter_visible_keeps_menus_with_no_permission_requirement() {
        let menus = vec![menu(1, None, 0)]; // no permissions attached
        let visible = filter_visible(menus, &[]);
        assert_eq!(visible.len(), 1);
    }

    #[test]
    fn filter_visible_keeps_menu_when_caller_has_a_matching_permission() {
        let menus = vec![menu_with_permissions(1, None, &["user.manage"])];
        let visible = filter_visible(menus, &["user.manage".to_string()]);
        assert_eq!(visible.len(), 1);
    }

    #[test]
    fn filter_visible_drops_menu_when_caller_lacks_every_required_permission() {
        let menus = vec![menu_with_permissions(1, None, &["user.manage"])];
        let visible = filter_visible(menus, &["audit.read".to_string()]);
        assert!(visible.is_empty());
    }

    #[test]
    fn filter_visible_drops_inactive_menus_regardless_of_permissions() {
        let mut m = menu(1, None, 0);
        m.is_active = false;
        let visible = filter_visible(vec![m], &[]);
        assert!(visible.is_empty());
    }

    // ---- build_tree + filter_visible combined (the actual visible_tree behavior) ----

    #[test]
    fn hiding_a_parent_also_hides_its_children_in_the_visible_tree() {
        let admin_only_parent = menu_with_permissions(1, None, &["admin.only"]);
        let child = menu(2, Some(1), 0); // no permission requirement of its own

        let visible = filter_visible(vec![admin_only_parent, child], &[]);
        let tree = build_tree(visible);

        // The child individually has no permission requirement, but its
        // parent does and the caller doesn't hold it -- the whole subtree
        // must disappear, not just the parent.
        assert!(tree.is_empty());
    }
}
