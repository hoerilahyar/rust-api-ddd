pub mod dto;
pub mod service;
pub mod service_impl;

pub use dto::{CreateRoleRequest, RoleResponse, SyncRolePermissionsRequest, UpdateRoleRequest};
pub use service::RoleService;
pub use service_impl::RoleServiceImpl;
