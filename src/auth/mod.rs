//! Authentication and session management

pub mod jwt;
pub mod middleware;
pub mod models;
pub mod session;

pub use jwt::{create_token, validate_token, Claims};
pub use middleware::{extract_user_from_request, AuthLayer};
pub use models::{User, UserRole};
pub use session::{Session, SessionManager};
