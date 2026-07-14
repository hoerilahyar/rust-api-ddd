use std::sync::Arc;

use chrono::{DateTime, Utc};
use redis::aio::ConnectionManager;
use sqlx::PgPool;

use crate::bootstrap::config::AppConfig;
use crate::modules::activity_log::application::{ActivityLogService, ActivityLogServiceImpl};
use crate::modules::activity_log::infrastructure::persistence::ActivityLogRepositoryPg;
use crate::modules::audit_log::application::{AuditLogService, AuditLogServiceImpl};
use crate::modules::audit_log::infrastructure::persistence::AuditLogRepositoryPg;
use crate::modules::auth::application::service::AuthService;
use crate::modules::auth::application::service_impl::AuthServiceImpl;
use crate::modules::auth::infrastructure::jwt_service::JwtService;
use crate::modules::auth::infrastructure::persistence::AuthRepositoryPg;
use crate::modules::file::application::{FileService, FileServiceImpl};
use crate::modules::file::infrastructure::persistence::FileRepositoryPg;
use crate::modules::file::infrastructure::storage::LocalFileStorage;
use crate::modules::menu::application::{MenuService, MenuServiceImpl};
use crate::modules::menu::infrastructure::persistence::MenuRepositoryPg;
use crate::modules::permission::application::{PermissionService, PermissionServiceImpl};
use crate::modules::permission::infrastructure::persistence::PermissionRepositoryPg;
use crate::modules::role::application::{RoleService, RoleServiceImpl};
use crate::modules::role::infrastructure::persistence::RoleRepositoryPg;
use crate::modules::setting::application::{SettingService, SettingServiceImpl};
use crate::modules::setting::infrastructure::persistence::SettingRepositoryPg;
use crate::modules::user::application::service::UserService;
use crate::modules::user::application::service_impl::UserServiceImpl;
use crate::modules::user::infrastructure::persistence::UserRepositoryPg;
use crate::modules::user_setting::application::{UserSettingService, UserSettingServiceImpl};
use crate::modules::user_setting::infrastructure::persistence::UserSettingRepositoryPg;
use crate::shared::cache::RedisCacheRepository;
use crate::shared::contracts::ActivityRecorder;

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
    pub audit_log_service: Arc<dyn AuditLogService>,
    pub activity_log_service: Arc<dyn ActivityLogService>,
    /// Injected into other modules' services so they can log CRUD/view
    /// activity without depending on `activity_log`'s persistence layer.
    pub activity_recorder: Arc<dyn ActivityRecorder>,
    pub menu_service: Arc<dyn MenuService>,
    pub setting_service: Arc<dyn SettingService>,
    pub user_setting_service: Arc<dyn UserSettingService>,
    pub file_service: Arc<dyn FileService>,
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
        let audit_log_repo: Arc<AuditLogRepositoryPg> =
            Arc::new(AuditLogRepositoryPg::new(db.clone()));
        let activity_log_repo: Arc<ActivityLogRepositoryPg> =
            Arc::new(ActivityLogRepositoryPg::new(db.clone()));
        let menu_repo: Arc<MenuRepositoryPg> = Arc::new(MenuRepositoryPg::new(db.clone()));
        let setting_repo: Arc<SettingRepositoryPg> = Arc::new(SettingRepositoryPg::new(db.clone()));
        let user_setting_repo: Arc<UserSettingRepositoryPg> =
            Arc::new(UserSettingRepositoryPg::new(db.clone()));
        let file_repo: Arc<FileRepositoryPg> = Arc::new(FileRepositoryPg::new(db.clone()));
        let file_storage = Arc::new(
            LocalFileStorage::new(config.storage.base_path.clone())
                .expect("failed to initialize local file storage directory"),
        );

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

        let audit_log_service: Arc<dyn AuditLogService> =
            Arc::new(AuditLogServiceImpl::new(audit_log_repo));

        let activity_log_service: Arc<dyn ActivityLogService> =
            Arc::new(ActivityLogServiceImpl::new(activity_log_repo.clone()));
        // Same Postgres repo implements both the read-only `ActivityLogRepository`
        // (above) and the write-only `ActivityRecorder` contract (below).
        let activity_recorder: Arc<dyn ActivityRecorder> = activity_log_repo;

        let menu_service: Arc<dyn MenuService> =
            Arc::new(MenuServiceImpl::new(menu_repo, cache.clone()));

        let setting_service: Arc<dyn SettingService> =
            Arc::new(SettingServiceImpl::new(setting_repo, cache.clone()));

        let user_setting_service: Arc<dyn UserSettingService> = Arc::new(
            UserSettingServiceImpl::new(user_setting_repo, cache.clone()),
        );

        let file_service: Arc<dyn FileService> = Arc::new(FileServiceImpl::new(
            file_repo,
            file_storage,
            config.storage.max_upload_bytes,
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
            audit_log_service,
            activity_log_service,
            activity_recorder,
            menu_service,
            setting_service,
            user_setting_service,
            file_service,
        }
    }
}
