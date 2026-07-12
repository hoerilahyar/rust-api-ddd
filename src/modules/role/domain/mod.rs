pub mod entity;
pub mod errors;
pub mod repository;
pub mod value_object;

pub use entity::{Permission, Role};
pub use errors::RoleDomainError;
pub use repository::RoleRepository;
pub use value_object::Name;
