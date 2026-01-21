pub mod jwt;

pub use jwt::{create_token, verify_token, AuthError, Claims, TOKEN_EXPIRY_SECS};

use serde::{Deserialize, Serialize};

/// 登录请求（username 字段语义为邮箱）
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

/// 注册请求（username 字段语义为邮箱）
#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
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

/// 注册响应
#[derive(Debug, Serialize)]
pub struct RegisterResponse {
    pub user: User,
}

use dashmap::DashMap;
use std::sync::OnceLock;

#[derive(Debug, Clone)]
struct RegisteredUser {
    password: String,
    user: User,
}

static REGISTERED_USERS: OnceLock<DashMap<String, RegisteredUser>> = OnceLock::new();

fn registered_users() -> &'static DashMap<String, RegisteredUser> {
    REGISTERED_USERS.get_or_init(DashMap::new)
}

fn normalize_email(email: &str) -> String {
    email.trim().to_ascii_lowercase()
}

fn is_valid_email(email: &str) -> bool {
    let e = email.trim();
    let at = e.find('@');
    match at {
        Some(i) if i > 0 && i + 1 < e.len() => e[i + 1..].contains('.'),
        _ => false,
    }
}

fn user_id_from_email(email: &str) -> String {
    format!(
        "u_{}",
        email
            .chars()
            .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
            .collect::<String>()
    )
}

/// 简单的用户注册（生产环境应使用数据库/加密存储密码）
pub fn register_user(username: &str, password: &str) -> Result<User, String> {
    let email = normalize_email(username);
    if !is_valid_email(&email) {
        return Err("invalid_email".to_string());
    }
    if password.is_empty() {
        return Err("invalid_password".to_string());
    }

    if registered_users().contains_key(&email) {
        return Err("user_already_exists".to_string());
    }

    let user = User {
        id: user_id_from_email(&email),
        username: email.clone(),
        display_name: None,
        roles: vec!["user".to_string()],
    };

    registered_users().insert(
        email,
        RegisteredUser {
            password: password.to_string(),
            user: user.clone(),
        },
    );

    Ok(user)
}

/// 简单的用户验证（生产环境应使用数据库）
pub fn validate_user(username: &str, password: &str) -> Option<User> {
    // 默认用户: admin/admin, user/user
    match (username, password) {
        ("admin", "admin") => {
            return Some(User {
                id: "u_admin".to_string(),
                username: "admin".to_string(),
                display_name: Some("Administrator".to_string()),
                roles: vec!["admin".to_string()],
            })
        }
        ("user", "user") => {
            return Some(User {
                id: "u_user".to_string(),
                username: "user".to_string(),
                display_name: Some("User".to_string()),
                roles: vec!["user".to_string()],
            })
        }
        _ => {}
    }

    let email = normalize_email(username);
    let reg = registered_users().get(&email)?;
    if reg.password == password {
        Some(reg.user.clone())
    } else {
        None
    }
}
