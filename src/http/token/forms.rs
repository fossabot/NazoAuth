//! Token 相关表单模型。
// 表单结构在多个 token 子模块之间共享。
use crate::http::prelude::*;

#[derive(Deserialize)]
pub(crate) struct TokenForm {
    pub(crate) grant_type: String,
    pub(crate) code: Option<String>,
    pub(crate) redirect_uri: Option<String>,
    pub(crate) code_verifier: Option<String>,
    pub(crate) refresh_token: Option<String>,
    pub(crate) scope: Option<String>,
    pub(crate) client_id: Option<String>,
    pub(crate) client_secret: Option<String>,
    pub(crate) audience: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct TokenOnlyForm {
    pub(crate) token: String,
    pub(crate) client_id: Option<String>,
    pub(crate) client_secret: Option<String>,
}
