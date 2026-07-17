use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use chrono::Utc;

use crate::modules::log_audit_trails::domain::entity::AuditTrailLog;
use crate::modules::user_profile::application::dto::UpsertUserProfileRequest;
use crate::modules::user_profile::application::service::UserProfileService;
use crate::modules::user_profile::domain::{UserProfile, UserProfileRepository};
use crate::shared::cache::{CacheRepository, RedisCacheRepository};
use crate::shared::context::current_request_context;
use crate::shared::contracts::AuditTrailRecorder;
use crate::shared::errors::AppError;

const ENTITY_TYPE: &str = "user_profile";

/// Fires an audit trail write in the background so a slow/unavailable audit
/// sink never blocks or fails the actual profile mutation. `user_id` is
/// both the subject and the actor here -- every method in this module is
/// self-service, scoped to the caller's own `Claims::sub` (see the trait
/// doc for the one admin-read exception).
fn spawn_audit_log(
    audit: Arc<dyn AuditTrailRecorder>,
    user_id: i32,
    action: &'static str,
    old_values: Option<&UserProfile>,
    new_values: Option<&UserProfile>,
) {
    let ctx = current_request_context();
    let log = AuditTrailLog {
        id: 0,
        user_id: Some(user_id),
        action: action.to_string(),
        entity_type: ENTITY_TYPE.to_string(),
        entity_id: Some(user_id.to_string()),
        old_values: old_values.and_then(|s| serde_json::to_value(s).ok()),
        new_values: new_values.and_then(|s| serde_json::to_value(s).ok()),
        ip_address: ctx.ip_address,
        user_agent: ctx.user_agent,
        description: Some(format!("user {user_id} profile update")),
        created_at: Utc::now(),
    };

    tokio::spawn(async move {
        if let Err(err) = audit.record_audit_trail_log(log).await {
            tracing::error!(error = ?err, user_id, action, "failed to record user profile audit trail log");
        }
    });
}

const CACHE_TTL: Duration = Duration::from_secs(300);

fn cache_key(user_id: i32) -> String {
    format!("user_profile:{user_id}")
}

/// An empty profile for a user who hasn't filled anything in yet. `GET
/// /me/profile` returns this instead of a 404 so a fresh account can load
/// its (blank) profile the same way an existing one does.
fn blank(user_id: i32) -> UserProfile {
    let now = Utc::now();
    UserProfile {
        id: 0,
        user_id,
        phone: None,
        address: None,
        city: None,
        country: None,
        postal_code: None,
        gender: None,
        date_of_birth: None,
        avatar_url: None,
        website: None,
        bio: None,
        created_at: now,
        updated_at: now,
    }
}

pub struct UserProfileServiceImpl {
    audit: Arc<dyn AuditTrailRecorder>,
    repo: Arc<dyn UserProfileRepository>,
    cache: Arc<RedisCacheRepository>,
}

impl UserProfileServiceImpl {
    pub fn new(
        audit: Arc<dyn AuditTrailRecorder>,
        repo: Arc<dyn UserProfileRepository>,
        cache: Arc<RedisCacheRepository>,
    ) -> Self {
        Self { audit, repo, cache }
    }
}

#[async_trait]
impl UserProfileService for UserProfileServiceImpl {
    async fn get(&self, user_id: i32) -> Result<UserProfile, AppError> {
        let ck = cache_key(user_id);
        if let Some(cached) = self.cache.get::<UserProfile>(&ck).await? {
            return Ok(cached);
        }

        let profile = match self.repo.find_by_user_id(user_id).await? {
            Some(p) => p,
            None => return Ok(blank(user_id)),
        };

        self.cache.set(&ck, &profile, CACHE_TTL).await?;
        Ok(profile)
    }

    async fn upsert(
        &self,
        user_id: i32,
        req: UpsertUserProfileRequest,
    ) -> Result<UserProfile, AppError> {
        let old = self.repo.find_by_user_id(user_id).await.ok().flatten();

        let profile = self
            .repo
            .upsert(
                user_id,
                req.phone.as_deref(),
                req.address.as_deref(),
                req.city.as_deref(),
                req.country.as_deref(),
                req.postal_code.as_deref(),
                req.gender.as_deref(),
                req.date_of_birth,
                req.avatar_url.as_deref(),
                req.website.as_deref(),
                req.bio.as_deref(),
            )
            .await?;

        spawn_audit_log(
            self.audit.clone(),
            user_id,
            "user_profile.upsert",
            old.as_ref(),
            Some(&profile),
        );

        self.cache.delete(&cache_key(user_id)).await?;
        Ok(profile)
    }

    async fn delete(&self, user_id: i32) -> Result<(), AppError> {
        let old = self.repo.find_by_user_id(user_id).await.ok().flatten();

        self.repo.delete(user_id).await?;

        spawn_audit_log(
            self.audit.clone(),
            user_id,
            "user_profile.delete",
            old.as_ref(),
            None,
        );

        self.cache.delete(&cache_key(user_id)).await?;
        Ok(())
    }
}
