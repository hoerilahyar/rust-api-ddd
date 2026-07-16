use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;

use chrono::Utc;

use crate::modules::audit_trail_log::domain::entity::AuditTrailLog;
use crate::modules::masters::application::{
    CreateMasterItemRequest, MasterItemService, UpdateMasterItemRequest,
};
use crate::modules::masters::domain::{ItemName, MasterItem, MasterItemRepository};
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
    master_id: i32,
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

fn cache_key(id: i32) -> String {
    format!("master_item:id:{id}")
}

pub struct MasterItemServiceImpl {
    audit: Arc<dyn AuditTrailRecorder>,
    repo: Arc<dyn MasterItemRepository>,
    cache: Arc<RedisCacheRepository>,
}

impl MasterItemServiceImpl {
    pub fn new(
        audit: Arc<dyn AuditTrailRecorder>,
        repo: Arc<dyn MasterItemRepository>,
        cache: Arc<RedisCacheRepository>,
    ) -> Self {
        Self { audit, repo, cache }
    }
}

#[async_trait]
impl MasterItemService for MasterItemServiceImpl {
    async fn get_by_id(&self, id: i32) -> Result<MasterItem, AppError> {
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
        Ok(item)
    }

    async fn update(
        &self,
        id: i32,
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
        Ok(item)
    }

    async fn delete(&self, id: i32, actor_id: i32) -> Result<(), AppError> {
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
        Ok(())
    }
}
