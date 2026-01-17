use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// JWT Claims
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,        // user_id
    pub username: String,
    pub roles: Vec<String>,
    pub exp: u64,           // 过期时间
    pub iat: u64,           // 签发时间
}

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("Invalid token")]
    InvalidToken,
    #[error("Token expired")]
    TokenExpired,
    #[error("Missing token")]
    MissingToken,
}

/// 获取 JWT 密钥（生产环境应从环境变量读取）
fn get_secret() -> Vec<u8> {
    std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "openrunner-dev-secret-change-in-production".to_string())
        .into_bytes()
}

/// Token 有效期（秒）
pub const TOKEN_EXPIRY_SECS: u64 = 3600 * 24; // 24 小时

/// 创建 JWT Token
pub fn create_token(user_id: &str, username: &str, roles: &[String]) -> Result<String, AuthError> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let claims = Claims {
        sub: user_id.to_string(),
        username: username.to_string(),
        roles: roles.to_vec(),
        iat: now,
        exp: now + TOKEN_EXPIRY_SECS,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(&get_secret()),
    )
    .map_err(|_| AuthError::InvalidToken)
}

/// 验证 JWT Token
pub fn verify_token(token: &str) -> Result<Claims, AuthError> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(&get_secret()),
        &Validation::default(),
    )
    .map_err(|e| {
        if e.to_string().contains("expired") {
            AuthError::TokenExpired
        } else {
            AuthError::InvalidToken
        }
    })?;

    Ok(token_data.claims)
}
