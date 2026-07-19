use std::time::Duration;

use serde::{Deserialize, Serialize};

// `CacheRepository` declares its `get`/`set` methods as generic over `T`,
// which makes the trait itself not object-safe -- the rest of this codebase
// always holds the concrete `RedisCacheRepository` (see every
// `*ServiceImpl`'s `cache: Arc<RedisCacheRepository>` field) rather than
// `Arc<dyn CacheRepository>`. These functions follow that same convention;
// the trait import below is only needed to bring its methods into scope.
use crate::shared::cache::{CacheRepository, RedisCacheRepository};
use crate::shared::contracts::UserReader;
use crate::shared::errors::AppError;

/// Live snapshot of a user's authorization state (active status, roles,
/// permissions) as of "right now" -- as opposed to `Claims`, which is a
/// snapshot as of "whenever the access token was issued".
///
/// `require_auth` fetches this on every request and overwrites the decoded
/// token's `roles`/`permissions`/`is_active` with it before the request
/// reaches a handler. That is what makes role/permission/menu changes take
/// effect immediately, on the very next request, instead of waiting for the
/// access token to expire or for the user to log out and back in.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthzSnapshot {
    pub is_active: bool,
    pub roles: Vec<String>,
    pub permissions: Vec<String>,
}

/// Safety-net TTL only. Every mutation that narrows or widens a user's
/// access is expected to call [`invalidate`] synchronously in the same
/// request that performs the change, so in the normal case a stale read
/// from this cache should never actually happen -- this TTL just bounds
/// the damage if an invalidation call is ever missed or fails.
const SNAPSHOT_TTL: Duration = Duration::from_secs(30);

fn cache_key(user_id: i32) -> String {
    format!("authz:user:{user_id}")
}

/// Cache-aside fetch of a user's current roles/permissions/active status.
/// Returns `Ok(None)` if the user no longer exists (e.g. hard-deleted).
pub async fn get_snapshot(
    cache: &RedisCacheRepository,
    user_reader: &dyn UserReader,
    user_id: i32,
) -> Result<Option<AuthzSnapshot>, AppError> {
    let key = cache_key(user_id);

    if let Some(cached) = cache.get::<AuthzSnapshot>(&key).await? {
        return Ok(Some(cached));
    }

    let Some(user) = user_reader.find_by_id(user_id).await? else {
        return Ok(None);
    };

    let snapshot = AuthzSnapshot {
        is_active: user.is_active,
        roles: user.roles,
        permissions: user.permissions,
    };

    cache.set(&key, &snapshot, SNAPSHOT_TTL).await?;
    Ok(Some(snapshot))
}

/// Call this the instant a user's roles/permissions/active status change
/// (role assigned/revoked, a role's permissions edited, account
/// deactivated/deleted, ...) so their very next request -- even one using
/// an access token minted minutes ago -- sees the update right away.
pub async fn invalidate(cache: &RedisCacheRepository, user_id: i32) -> Result<(), AppError> {
    cache.delete(&cache_key(user_id)).await
}
