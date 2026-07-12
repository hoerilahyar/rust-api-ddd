pub mod dto;
pub mod service;
pub mod service_impl;

pub use dto::{AssignPermissionRequest, CreateRoleRequest, RoleResponse, UpdateRoleRequest};
pub use service::RoleService;
pub use service_impl::RoleServiceImpl;
