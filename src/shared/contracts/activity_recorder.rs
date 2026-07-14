use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::shared::errors::AppError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Activity {
    Login,
    Logout,
    View,
    List,
    Search,
    Create,
    Update,
    Delete,
    Upload,
    Download,
    Export,
    Import,
    Assign,
    Unassign,
    Enable,
    Disable,
}

impl Activity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Activity::Login => "LOGIN",
            Activity::Logout => "LOGOUT",
            Activity::View => "VIEW",
            Activity::List => "LIST",
            Activity::Search => "SEARCH",
            Activity::Create => "CREATE",
            Activity::Update => "UPDATE",
            Activity::Delete => "DELETE",
            Activity::Upload => "UPLOAD",
            Activity::Download => "DOWNLOAD",
            Activity::Export => "EXPORT",
            Activity::Import => "IMPORT",
            Activity::Assign => "ASSIGN",
            Activity::Unassign => "UNASSIGN",
            Activity::Enable => "ENABLE",
            Activity::Disable => "DISABLE",
        }
    }

    pub fn from_str_or_view(value: &str) -> Self {
        match value {
            "LOGIN" => Activity::Login,
            "LOGOUT" => Activity::Logout,
            "VIEW" => Activity::View,
            "LIST" => Activity::List,
            "SEARCH" => Activity::Search,
            "CREATE" => Activity::Create,
            "UPDATE" => Activity::Update,
            "DELETE" => Activity::Delete,
            "UPLOAD" => Activity::Upload,
            "DOWNLOAD" => Activity::Download,
            "EXPORT" => Activity::Export,
            "IMPORT" => Activity::Import,
            "ASSIGN" => Activity::Assign,
            "UNASSIGN" => Activity::Unassign,
            "ENABLE" => Activity::Enable,
            "DISABLE" => Activity::Disable,
            _ => Activity::View,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum MethodRequest {
    Get,
    Post,
    Put,
    Patch,
    Delete,
}

impl MethodRequest {
    pub fn as_str(&self) -> &'static str {
        match self {
            MethodRequest::Get => "GET",
            MethodRequest::Post => "POST",
            MethodRequest::Put => "PUT",
            MethodRequest::Patch => "PATCH",
            MethodRequest::Delete => "DELETE",
        }
    }

    pub fn from_str_or_get(value: &str) -> Self {
        match value {
            "GET" => MethodRequest::Get,
            "POST" => MethodRequest::Post,
            "PUT" => MethodRequest::Put,
            "PATCH" => MethodRequest::Patch,
            "DELETE" => MethodRequest::Delete,
            _ => MethodRequest::Get,
        }
    }
}

/// Which module/bounded-context an activity happened in. Kept as a plain
/// string on the wire (see `as_str`) so adding a new module never requires a
/// migration -- only widening this enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Module {
    Auth,
    User,
    Role,
    Permission,
    Menu,
    Setting,
    UserSetting,
    Audit,
    ActivityLog,
    File,
}

impl Module {
    pub fn as_str(&self) -> &'static str {
        match self {
            Module::Auth => "Auth",
            Module::User => "User",
            Module::Role => "Role",
            Module::Permission => "Permission",
            Module::Menu => "Menu",
            Module::Setting => "Setting",
            Module::UserSetting => "UserSetting",
            Module::Audit => "Audit",
            Module::ActivityLog => "Log",
            Module::File => "File",
        }
    }

    pub fn from_str_or_log(value: &str) -> Self {
        match value {
            "Auth" => Module::Auth,
            "User" => Module::User,
            "Role" => Module::Role,
            "Permission" => Module::Permission,
            "Menu" => Module::Menu,
            "Setting" => Module::Setting,
            "UserSetting" => Module::UserSetting,
            "Audit" => Module::Audit,
            "Log" => Module::ActivityLog,
            "File" => Module::File,
            _ => Module::ActivityLog,
        }
    }
}

/// Input for writing one row to `activity_logs`. Deliberately separate from
/// the `activity_log` module's own read-side `ActivityLog` entity (mirrors
/// how `LoginAttempt` is separate from `audit::domain::LoginLog`), so any
/// module can log an action without depending on `activity_log`'s
/// persistence types.
#[derive(Debug, Clone)]
pub struct RecordActivity {
    pub user_id: Option<i32>,
    pub activity: Activity,
    pub module: Module,
    pub resource_type: Option<String>,
    pub resource_id: Option<String>,
    pub method: MethodRequest,
    pub path: String,
    pub description: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub status_code: Option<i16>,
    pub trace_id: Option<Uuid>,
}

/// Cross-cutting write contract. Lives in `shared` (not in `activity_log`)
/// so that any module can append to `activity_logs` without depending on
/// `activity_log`'s persistence layer directly -- the same reasoning as
/// `AuditRecorder` for `user_login_logs`.
#[async_trait]
pub trait ActivityRecorder: Send + Sync {
    async fn record_activity(&self, activity: RecordActivity) -> Result<(), AppError>;
}
