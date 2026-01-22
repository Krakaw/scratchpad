//! Authentication and session management

pub mod session;
pub mod jwt;
pub mod middleware;
pub mod models;

pub use jwt::{create_token, validate_token, Claims};
pub use middleware::{AuthLayer, extract_user_from_request};
pub use models::{User, UserRole};
pub use session::{SessionManager, Session};
