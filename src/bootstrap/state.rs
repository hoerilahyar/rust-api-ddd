use std::sync::Arc;

use chrono::{DateTime, Utc};
use redis::aio::ConnectionManager;
use sqlx::PgPool;

use crate::bootstrap::config::AppConfig;
use crate::modules::auth::application::service::AuthService;
use crate::modules::auth::application::service_impl::AuthServiceImpl;
use crate::modules::auth::infrastructure::jwt_service::JwtService;
use crate::modules::auth::infrastructure::persistence::AuthRepositoryPg;
use crate::modules::permission::application::{PermissionService, PermissionServiceImpl};
use crate::modules::permission::infrastructure::persistence::PermissionRepositoryPg;
use crate::modules::role::application::{RoleService, RoleServiceImpl};
use crate::modules::role::infrastructure::persistence::RoleRepositoryPg;
use crate::modules::user::application::service::UserService;
use crate::modules::user::application::service_impl::UserServiceImpl;
use crate::modules::user::infrastructure::persistence::UserRepositoryPg;
use crate::shared::cache::RedisCacheRepository;

/// Shared application state injected into every handler via `State<AppState>`.
/// Cheap to clone: everything inside is either a connection pool/manager or
/// an `Arc`.
#[derive(Clone)]
pub struct AppState {
    pub started_at: DateTime<Utc>,
    pub config: Arc<AppConfig>,
    pub db: PgPool,
    pub redis: ConnectionManager,
    pub jwt: Arc<JwtService>,
    pub user_service: Arc<dyn UserService>,
    pub auth_service: Arc<dyn AuthService>,
    pub role_service: Arc<dyn RoleService>,
    pub permission_service: Arc<dyn PermissionService>,
}

impl AppState {
    /// Wires every module's dependencies together: repositories -> services,
    /// with the concrete Postgres/Redis implementations behind the
    /// `UserService`/`AuthService` trait objects.
    pub fn new(config: AppConfig, db: PgPool, redis: ConnectionManager) -> Self {
        let config = Arc::new(config);
        let jwt = Arc::new(JwtService::new(&config.jwt));

        let cache = Arc::new(RedisCacheRepository::new(redis.clone()));

        let user_repo: Arc<UserRepositoryPg> = Arc::new(UserRepositoryPg::new(db.clone()));
        let auth_repo: Arc<AuthRepositoryPg> = Arc::new(AuthRepositoryPg::new(db.clone()));
        let role_repo: Arc<RoleRepositoryPg> = Arc::new(RoleRepositoryPg::new(db.clone()));
        let permission_repo: Arc<PermissionRepositoryPg> =
            Arc::new(PermissionRepositoryPg::new(db.clone()));

        let user_service: Arc<dyn UserService> =
            Arc::new(UserServiceImpl::new(user_repo.clone(), cache.clone()));

        let auth_service: Arc<dyn AuthService> = Arc::new(AuthServiceImpl::new(
            auth_repo.clone(),
            user_repo, // implements shared::contracts::UserReader
            auth_repo, // implements shared::contracts::AuditRecorder
            jwt.clone(),
        ));

        let role_service: Arc<dyn RoleService> =
            Arc::new(RoleServiceImpl::new(role_repo.clone(), cache.clone()));

        let permission_service: Arc<dyn PermissionService> = Arc::new(PermissionServiceImpl::new(
            permission_repo.clone(),
            cache.clone(),
        ));

        Self {
            started_at: Utc::now(),
            config,
            db,
            redis,
            jwt,
            user_service,
            auth_service,
            role_service,
            permission_service,
        }
    }
}
