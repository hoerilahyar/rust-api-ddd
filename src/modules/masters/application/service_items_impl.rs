use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;

use chrono::Utc;

use crate::modules::log_audit_trails::domain::entity::AuditTrailLog;
use crate::modules::masters::application::{
    CreateMasterItemRequest, MasterItemService, UpdateMasterItemRequest,
};
use crate::modules::masters::domain::{
    ItemName, MasterGroupRepository, MasterItem, MasterItemRepository,
};
use crate::shared::cache::{CacheRepository, RedisCacheRepository};
use crate::shared::context::current_request_context;
use crate::shared::contracts::AuditTrailRecorder;
use crate::shared::domain::PaginationParams;
use crate::shared::errors::AppError;

const ENTITY_TYPE: &str = "master_item";

/// Fires an audit trail write in the background so a slow/unavailable audit
/// sink never blocks or fails the actual master item mutation.
fn spawn_audit_log(
    audit: Arc<dyn AuditTrailRecorder>,
    actor_id: i32,
    action: &'static str,
    master_id: i64,
    old_values: Option<&MasterItem>,
    new_values: Option<&MasterItem>,
) {
    let ctx = current_request_context();
    let log = AuditTrailLog {
        id: 0,
        user_id: Some(actor_id),
        action: action.to_string(),
        entity_type: ENTITY_TYPE.to_string(),
        entity_id: Some(master_id.to_string()),
        old_values: old_values.and_then(|r| serde_json::to_value(r).ok()),
        new_values: new_values.and_then(|r| serde_json::to_value(r).ok()),
        ip_address: ctx.ip_address,
        user_agent: ctx.user_agent,
        description: Some(format!("master item id {master_id}")),
        created_at: Utc::now(),
    };

    tokio::spawn(async move {
        if let Err(err) = audit.record_audit_trail_log(log).await {
            tracing::error!(error = ?err, master_id, action, "failed to record master item audit trail log");
        }
    });
}

const CACHE_TTL: Duration = Duration::from_secs(300);

fn cache_key(id: i64) -> String {
    format!("master_item:id:{id}")
}

/// Mirrors `service_group_impl::cache_key` so item mutations can bust the
/// parent group's cached response (which embeds the full items list) --
/// see the doc comment on `invalidate_group_cache` below for why this
/// matters.
fn group_cache_key(group_id: i64) -> String {
    format!("master_group:id:{group_id}")
}

pub struct MasterItemServiceImpl {
    audit: Arc<dyn AuditTrailRecorder>,
    repo: Arc<dyn MasterItemRepository>,
    cache: Arc<RedisCacheRepository>,
    /// Used to validate a `group_id` exists (and isn't soft-deleted) before
    /// an item is created under it, so a bad/stale group id surfaces as a
    /// clean 404 instead of either an FK violation or, worse, silently
    /// succeeding and orphaning the item under a group that will never
    /// list it -- `MasterGroupRepository::find_by_id` already excludes
    /// soft-deleted rows.
    group_repo: Arc<dyn MasterGroupRepository>,
}

impl MasterItemServiceImpl {
    pub fn new(
        audit: Arc<dyn AuditTrailRecorder>,
        repo: Arc<dyn MasterItemRepository>,
        cache: Arc<RedisCacheRepository>,
        group_repo: Arc<dyn MasterGroupRepository>,
    ) -> Self {
        Self {
            audit,
            repo,
            cache,
            group_repo,
        }
    }

    /// `MasterGroupServiceImpl::get_by_id` caches the group's full response
    /// -- including its embedded `items` list -- for `CACHE_TTL`. Every
    /// item mutation here changes what that cached response should contain,
    /// so it must be busted here too; otherwise the group's item list stays
    /// stale for up to 5 minutes after an item is created, edited, or
    /// deleted, even though `master_item:id:*` itself is fresh.
    async fn invalidate_group_cache(&self, group_id: i64) -> Result<(), AppError> {
        self.cache.delete(&group_cache_key(group_id)).await
    }
}

#[async_trait]
impl MasterItemService for MasterItemServiceImpl {
    async fn get_by_id(&self, id: i64) -> Result<MasterItem, AppError> {
        let key = cache_key(id);
        if let Some(cached) = self.cache.get::<MasterItem>(&key).await? {
            return Ok(cached);
        }

        let item = self
            .repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound("master item not found".to_string()))?;

        self.cache.set(&key, &item, CACHE_TTL).await?;
        Ok(item)
    }

    async fn list(
        &self,
        pagination: &PaginationParams,
    ) -> Result<(Vec<MasterItem>, i64), AppError> {
        self.repo.list(pagination).await
    }

    async fn create(
        &self,
        req: CreateMasterItemRequest,
        actor_id: i32,
    ) -> Result<MasterItem, AppError> {
        ItemName::parse(&req.name)?;

        self.group_repo
            .find_by_id(req.group_id)
            .await?
            .ok_or_else(|| AppError::NotFound("master group not found".to_string()))?;

        if self
            .repo
            .find_by_group_and_code(req.group_id, &req.code)
            .await?
            .is_some()
        {
            return Err(AppError::Conflict(
                "code is already registered in this group".to_string(),
            ));
        }

        let item = self
            .repo
            .create(
                req.group_id,
                &req.code,
                &req.name,
                req.description.as_deref(),
                req.extra.clone(),
                req.sort_order,
            )
            .await?;

        spawn_audit_log(
            self.audit.clone(),
            actor_id,
            "master_item.create",
            item.id,
            None,
            Some(&item),
        );
        self.invalidate_group_cache(item.group_id).await?;
        Ok(item)
    }

    async fn update(
        &self,
        id: i64,
        req: UpdateMasterItemRequest,
        actor_id: i32,
    ) -> Result<MasterItem, AppError> {
        ItemName::parse(&req.name)?;

        let existing = self
            .repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound("master item not found".to_string()))?;

        if let Some(dup) = self
            .repo
            .find_by_group_and_code(existing.group_id, &req.code)
            .await?
        {
            if dup.id != id {
                return Err(AppError::Conflict(
                    "code is already registered in this group".to_string(),
                ));
            }
        }

        let item = self
            .repo
            .update(
                id,
                Some(req.code.as_str()),
                Some(req.name.as_str()),
                req.description.as_deref(),
                req.extra.clone(),
                req.sort_order,
                Some(&req.is_active),
            )
            .await?;

        spawn_audit_log(
            self.audit.clone(),
            actor_id,
            "master_item.update",
            id,
            Some(&existing),
            Some(&item),
        );

        self.cache.delete(&cache_key(id)).await?;
        self.invalidate_group_cache(item.group_id).await?;
        Ok(item)
    }

    async fn delete(&self, id: i64, actor_id: i32) -> Result<(), AppError> {
        let existing = self
            .repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound("master item not found".to_string()))?;

        self.repo.delete(id).await?;

        spawn_audit_log(
            self.audit.clone(),
            actor_id,
            "master_item.delete",
            id,
            Some(&existing),
            None,
        );

        self.cache.delete(&cache_key(id)).await?;
        self.invalidate_group_cache(existing.group_id).await?;
        Ok(())
    }
}
