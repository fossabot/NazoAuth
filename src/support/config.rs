//! 启动配置辅助函数。
// 只放环境值归一化和随机 token 这类无外部状态的小工具。

use super::prelude::*;

pub(crate) fn normalize_database_url(url: &str) -> String {
    url.replace("postgresql+psycopg://", "postgresql://")
}

pub(crate) fn random_urlsafe_token() -> String {
    URL_SAFE_NO_PAD.encode(rand::random::<[u8; 32]>())
}
