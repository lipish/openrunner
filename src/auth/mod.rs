pub mod jwt;

pub use jwt::{create_token, verify_token, Claims, AuthError, TOKEN_EXPIRY_SECS};

use serde::{Deserialize, Serialize};

/// 登录请求
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

/// 用户信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub username: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(default)]
    pub roles: Vec<String>,
}

/// 登录响应
#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,
    pub user: User,
}

/// 简单的用户验证（生产环境应使用数据库）
pub fn validate_user(username: &str, password: &str) -> Option<User> {
    // TODO: 替换为真实的用户验证
    // 默认用户: admin/admin, user/user
    match (username, password) {
        ("admin", "admin") => Some(User {
            id: "u_admin".to_string(),
            username: "admin".to_string(),
            display_name: Some("Administrator".to_string()),
            roles: vec!["admin".to_string()],
        }),
        ("user", "user") => Some(User {
            id: "u_user".to_string(),
            username: "user".to_string(),
            display_name: Some("User".to_string()),
            roles: vec!["user".to_string()],
        }),
        _ => None,
    }
}
