use std::sync::Arc;

use chrono::{DateTime, Utc};
use redis::aio::ConnectionManager;
use sqlx::PgPool;

use crate::bootstrap::config::AppConfig;
use crate::modules::auth::application::service::AuthService;
use crate::modules::auth::application::service_impl::AuthServiceImpl;
use crate::modules::auth::infrastructure::jwt_service::JwtService;
use crate::modules::auth::infrastructure::persistence::AuthRepositoryPg;
use crate::modules::file::application::{FileService, FileServiceImpl};
use crate::modules::file::infrastructure::persistence::FileRepositoryPg;
use crate::modules::file::infrastructure::storage::LocalFileStorage;
use crate::modules::log_activities::application::{ActivityLogService, ActivityLogServiceImpl};
use crate::modules::log_activities::infrastructure::persistence::ActivityLogRepositoryPg;
use crate::modules::log_audit_auths::application::{AuditAuthLogService, AuditAuthLogServiceImpl};
use crate::modules::log_audit_auths::infrastructure::persistence::AuditAuthLogRepositoryPg;
use crate::modules::log_audit_trails::application::{
    AuditTrailLogService, AuditTrailLogServiceImpl,
};
use crate::modules::log_audit_trails::infrastructure::persistence::AuditTrailLogRepositoryPg;
use crate::modules::masters::application::{
    MasterGroupService, MasterGroupServiceImpl, MasterItemService, MasterItemServiceImpl,
};
use crate::modules::masters::infrastructure::persistence::{
    MasterGroupRepositoryPg, MasterItemRepositoryPg,
};
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
use crate::modules::user_profile::application::{UserProfileService, UserProfileServiceImpl};
use crate::modules::user_profile::infrastructure::persistence::UserProfileRepositoryPg;
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
    pub audit_auth_log_service: Arc<dyn AuditAuthLogService>,
    pub audit_trail_log_service: Arc<dyn AuditTrailLogService>,
    pub activity_log_service: Arc<dyn ActivityLogService>,
    /// Injected into other modules' services so they can log CRUD/view
    /// activity without depending on `activity_log`'s persistence layer.
    pub activity_recorder: Arc<dyn ActivityRecorder>,
    pub menu_service: Arc<dyn MenuService>,
    pub setting_service: Arc<dyn SettingService>,
    pub user_setting_service: Arc<dyn UserSettingService>,
    pub user_profile_service: Arc<dyn UserProfileService>,
    pub file_service: Arc<dyn FileService>,
    pub master_group_service: Arc<dyn MasterGroupService>,
    pub master_item_service: Arc<dyn MasterItemService>,
}

impl AppState {
    /// Wires every module's dependencies together: repositories -> services,
    /// with the concrete Postgres/Redis implementations behind the
    /// `UserService`/`AuthService` trait objects.
    pub fn new(config: AppConfig, db: PgPool, redis: ConnectionManager) -> Self {
        let config = Arc::new(config);
        let jwt = Arc::new(JwtService::new(&config.jwt));

        let cache = Arc::new(RedisCacheRepository::new(redis.clone()));

        // ===========================================
        // ============== REPOSITORIES ===============
        // ===========================================
        let user_repo: Arc<UserRepositoryPg> = Arc::new(UserRepositoryPg::new(db.clone()));
        let auth_repo: Arc<AuthRepositoryPg> = Arc::new(AuthRepositoryPg::new(db.clone()));
        let role_repo: Arc<RoleRepositoryPg> = Arc::new(RoleRepositoryPg::new(db.clone()));
        let permission_repo: Arc<PermissionRepositoryPg> =
            Arc::new(PermissionRepositoryPg::new(db.clone()));
        let audit_auth_log_repo: Arc<AuditAuthLogRepositoryPg> =
            Arc::new(AuditAuthLogRepositoryPg::new(db.clone()));
        let audit_trail_log_repo: Arc<AuditTrailLogRepositoryPg> =
            Arc::new(AuditTrailLogRepositoryPg::new(db.clone()));
        let activity_log_repo: Arc<ActivityLogRepositoryPg> =
            Arc::new(ActivityLogRepositoryPg::new(db.clone()));
        let menu_repo: Arc<MenuRepositoryPg> = Arc::new(MenuRepositoryPg::new(db.clone()));
        let setting_repo: Arc<SettingRepositoryPg> = Arc::new(SettingRepositoryPg::new(db.clone()));
        let user_setting_repo: Arc<UserSettingRepositoryPg> =
            Arc::new(UserSettingRepositoryPg::new(db.clone()));
        let user_profile_repo: Arc<UserProfileRepositoryPg> =
            Arc::new(UserProfileRepositoryPg::new(db.clone()));
        let file_repo: Arc<FileRepositoryPg> = Arc::new(FileRepositoryPg::new(db.clone()));
        let file_storage = Arc::new(
            LocalFileStorage::new(config.storage.base_path.clone())
                .expect("failed to initialize local file storage directory"),
        );
        let master_group_repo: Arc<MasterGroupRepositoryPg> =
            Arc::new(MasterGroupRepositoryPg::new(db.clone()));
        let master_item_repo: Arc<MasterItemRepositoryPg> =
            Arc::new(MasterItemRepositoryPg::new(db.clone()));

        // ===========================================
        // ================ SERVICES =================
        // ===========================================
        let user_service: Arc<dyn UserService> = Arc::new(UserServiceImpl::new(
            audit_trail_log_repo.clone(),
            user_repo.clone(),
            cache.clone(),
        ));

        let auth_service: Arc<dyn AuthService> = Arc::new(AuthServiceImpl::new(
            auth_repo.clone(),
            user_repo,
            audit_auth_log_repo.clone(),
            jwt.clone(),
        ));

        let role_service: Arc<dyn RoleService> = Arc::new(RoleServiceImpl::new(
            audit_trail_log_repo.clone(),
            role_repo.clone(),
            cache.clone(),
        ));

        let permission_service: Arc<dyn PermissionService> = Arc::new(PermissionServiceImpl::new(
            audit_trail_log_repo.clone(),
            permission_repo.clone(),
            cache.clone(),
        ));

        let audit_auth_log_service: Arc<dyn AuditAuthLogService> =
            Arc::new(AuditAuthLogServiceImpl::new(audit_auth_log_repo));

        let audit_trail_log_service: Arc<dyn AuditTrailLogService> =
            Arc::new(AuditTrailLogServiceImpl::new(audit_trail_log_repo.clone()));

        let activity_log_service: Arc<dyn ActivityLogService> =
            Arc::new(ActivityLogServiceImpl::new(activity_log_repo.clone()));
        // Same Postgres repo implements both the read-only `ActivityLogRepository`
        // (above) and the write-only `ActivityRecorder` contract (below).
        let activity_recorder: Arc<dyn ActivityRecorder> = activity_log_repo;

        let menu_service: Arc<dyn MenuService> = Arc::new(MenuServiceImpl::new(
            audit_trail_log_repo.clone(),
            menu_repo,
            cache.clone(),
        ));

        let setting_service: Arc<dyn SettingService> = Arc::new(SettingServiceImpl::new(
            audit_trail_log_repo.clone(),
            setting_repo,
            cache.clone(),
        ));

        let user_setting_service: Arc<dyn UserSettingService> =
            Arc::new(UserSettingServiceImpl::new(
                audit_trail_log_repo.clone(),
                user_setting_repo,
                cache.clone(),
            ));

        let user_profile_service: Arc<dyn UserProfileService> =
            Arc::new(UserProfileServiceImpl::new(
                audit_trail_log_repo.clone(),
                user_profile_repo,
                cache.clone(),
            ));

        let file_service: Arc<dyn FileService> = Arc::new(FileServiceImpl::new(
            audit_trail_log_repo.clone(),
            file_repo,
            file_storage,
            config.storage.max_upload_bytes,
        ));

        let master_group_service: Arc<dyn MasterGroupService> =
            Arc::new(MasterGroupServiceImpl::new(
                audit_trail_log_repo.clone(),
                master_group_repo,
                cache.clone(),
            ));

        let master_item_service: Arc<dyn MasterItemService> = Arc::new(MasterItemServiceImpl::new(
            audit_trail_log_repo.clone(),
            master_item_repo,
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
            audit_auth_log_service,
            audit_trail_log_service,
            activity_log_service,
            activity_recorder,
            menu_service,
            setting_service,
            user_setting_service,
            user_profile_service,
            file_service,
            master_group_service,
            master_item_service,
        }
    }
}
