use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;

use crate::modules::menu::application::dto::{CreateMenuRequest, MenuTreeNode, UpdateMenuRequest};
use crate::modules::menu::application::service::MenuService;
use crate::modules::menu::domain::{Menu, MenuDomainError, MenuRepository};
use crate::shared::cache::{CacheRepository, RedisCacheRepository};
use crate::shared::domain::PaginationParams;
use crate::shared::errors::AppError;

const CACHE_TTL: Duration = Duration::from_secs(300);

fn cache_key(id: i32) -> String {
    format!("menu:id:{id}")
}

pub struct MenuServiceImpl {
    repo: Arc<dyn MenuRepository>,
    cache: Arc<RedisCacheRepository>,
}

impl MenuServiceImpl {
    pub fn new(repo: Arc<dyn MenuRepository>, cache: Arc<RedisCacheRepository>) -> Self {
        Self { repo, cache }
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
    roots.into_iter().map(|r| attach(r, &mut children)).collect()
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

    async fn visible_tree(&self, user_permissions: &[String]) -> Result<Vec<MenuTreeNode>, AppError> {
        let all = self.repo.list_all().await?;
        Ok(build_tree(filter_visible(all, user_permissions)))
    }

    async fn create(&self, req: CreateMenuRequest) -> Result<Menu, AppError> {
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

        Ok(menu)
    }

    async fn update(&self, id: i32, req: UpdateMenuRequest) -> Result<Menu, AppError> {
        if let Some(Some(new_parent)) = req.parent_id {
            let all = self.repo.list_all().await?;
            if !all.iter().any(|m| m.id == new_parent) {
                return Err(MenuDomainError::ParentNotFound.into());
            }
            if would_create_cycle(id, new_parent, &all) {
                return Err(MenuDomainError::CircularParent.into());
            }
        }

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

        self.cache.delete(&cache_key(id)).await?;
        Ok(menu)
    }

    async fn delete(&self, id: i32) -> Result<(), AppError> {
        self.repo.delete(id).await?;
        self.cache.delete(&cache_key(id)).await?;
        Ok(())
    }

    async fn assign_permission(&self, menu_id: i32, permission_name: &str) -> Result<(), AppError> {
        let (permission_id, _) = self
            .repo
            .find_permission_by_name(permission_name)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("permission '{permission_name}' not found")))?;

        self.repo.assign_permission(menu_id, permission_id).await?;
        self.cache.delete(&cache_key(menu_id)).await?;
        Ok(())
    }

    async fn revoke_permission(&self, menu_id: i32, permission_name: &str) -> Result<(), AppError> {
        let (permission_id, _) = self
            .repo
            .find_permission_by_name(permission_name)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("permission '{permission_name}' not found")))?;

        self.repo.revoke_permission(menu_id, permission_id).await?;
        self.cache.delete(&cache_key(menu_id)).await?;
        Ok(())
    }
}
