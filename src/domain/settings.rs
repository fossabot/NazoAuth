//! 环境配置读取与默认值。
// 配置只在启动阶段读取，运行期通过 AppState 共享不可变快照。
use std::{env, path::PathBuf};

/// OAuth 服务的运行参数。
#[derive(Clone)]
pub(crate) struct Settings {
    pub(crate) issuer: String,
    pub(crate) frontend_base_url: String,
    pub(crate) cors_allowed_origins: Vec<String>,
    pub(crate) default_audience: String,
    pub(crate) session_cookie_name: String,
    pub(crate) csrf_cookie_name: String,
    pub(crate) session_ttl_seconds: u64,
    pub(crate) auth_code_ttl_seconds: u64,
    pub(crate) access_token_ttl_seconds: i64,
    pub(crate) id_token_ttl_seconds: i64,
    pub(crate) refresh_token_ttl_seconds: i64,
    pub(crate) avatar_max_bytes: usize,
    pub(crate) client_delivery_ttl_seconds: u64,
    pub(crate) avatar_storage_dir: PathBuf,
    pub(crate) jwk_keys_dir: PathBuf,
}

impl Settings {
    /// 从环境变量构造设置；未提供时使用本地开发默认值。
    pub(crate) fn from_env() -> Self {
        Self {
            issuer: env::var("ISSUER").unwrap_or_else(|_| "http://127.0.0.1:8000".into()),
            frontend_base_url: env::var("FRONTEND_BASE_URL")
                .unwrap_or_else(|_| "http://127.0.0.1:3000".into()),
            cors_allowed_origins: env::var("CORS_ALLOWED_ORIGINS")
                .ok()
                .map(|value| {
                    value
                        .split(',')
                        .map(str::trim)
                        .filter(|value| !value.is_empty())
                        .map(ToOwned::to_owned)
                        .collect()
                })
                .filter(|values: &Vec<String>| !values.is_empty())
                .unwrap_or_else(|| vec!["http://127.0.0.1:3000".into()]),
            default_audience: env::var("DEFAULT_AUDIENCE")
                .unwrap_or_else(|_| "resource://default".into()),
            session_cookie_name: env::var("SESSION_COOKIE_NAME")
                .unwrap_or_else(|_| "nazo_oauth_session".into()),
            csrf_cookie_name: env::var("CSRF_COOKIE_NAME")
                .unwrap_or_else(|_| "nazo_oauth_csrf".into()),
            session_ttl_seconds: env::var("SESSION_TTL_SECONDS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(28_800),
            auth_code_ttl_seconds: env::var("AUTH_CODE_TTL_SECONDS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(300),
            access_token_ttl_seconds: env::var("ACCESS_TOKEN_TTL_SECONDS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(300),
            id_token_ttl_seconds: env::var("ID_TOKEN_TTL_SECONDS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(600),
            refresh_token_ttl_seconds: env::var("REFRESH_TOKEN_TTL_SECONDS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(2_592_000),
            avatar_max_bytes: env::var("AVATAR_MAX_BYTES")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(2_097_152),
            client_delivery_ttl_seconds: env::var("CLIENT_DELIVERY_TTL_SECONDS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(86_400),
            avatar_storage_dir: env::var("AVATAR_STORAGE_DIR")
                .map(PathBuf::from)
                .unwrap_or_else(|_| PathBuf::from("runtime/avatars")),
            jwk_keys_dir: env::var("JWK_KEYS_DIR")
                .map(PathBuf::from)
                .unwrap_or_else(|_| PathBuf::from("runtime/keys")),
        }
    }
}
