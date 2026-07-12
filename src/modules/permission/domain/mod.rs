pub mod entity;
pub mod errors;
pub mod repository;
pub mod value_object;

pub use entity::Permission;
pub use errors::PermissionDomainError;
pub use repository::PermissionRepository;
pub use value_object::Name;
