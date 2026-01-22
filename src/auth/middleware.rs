//! Authentication middleware and extractors

use crate::auth::{Claims, validate_token};
use crate::error::{Error, Result};
use axum::{
    extract::Request,
    middleware::Next,
    response::Response,
};

/// Extract user claims from request
pub fn extract_user_from_request(req: &Request) -> Result<Claims> {
    // Try to get token from Authorization header
    if let Some(auth_header) = req.headers().get("Authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if let Some(token) = auth_str.strip_prefix("Bearer ") {
                return validate_token(token);
            }
        }
    }

    // Try to get token from cookie
    if let Some(cookie_header) = req.headers().get("Cookie") {
        if let Ok(cookie_str) = cookie_header.to_str() {
            for cookie in cookie_str.split(';') {
                if let Some(token) = cookie.trim().strip_prefix("scratchpad_token=") {
                    return validate_token(token);
                }
            }
        }
    }

    Err(Error::Config("No valid authentication token found".to_string()))
}

/// Middleware for requiring authentication
pub async fn require_auth(req: Request, next: Next) -> std::result::Result<Response, Error> {
    // Try to extract claims - this will fail if no valid token
    let _claims = extract_user_from_request(&req)?;
    Ok(next.run(req).await)
}

/// Middleware layer for authentication
#[derive(Clone)]
pub struct AuthLayer;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_user_no_token() {
        use axum::http::Request;
        
        let req = Request::builder()
            .method("GET")
            .uri("/")
            .body(axum::body::Body::empty())
            .unwrap();

        let result = extract_user_from_request(&req);
        assert!(result.is_err());
    }
}

