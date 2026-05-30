//! 邮箱验证码发送端点。
// 当前端点负责生成和保存验证码；邮件投递可由独立投递服务接入。
use crate::http::prelude::*;

const EMAIL_CODE_TTL_SECONDS: u64 = 900;

#[derive(Deserialize)]
pub(crate) struct SendCodeRequest {
    email: String,
}

/// 生成并保存注册邮箱验证码。
pub(crate) async fn send_code(
    state: Data<AppState>,
    Json(payload): Json<SendCodeRequest>,
) -> HttpResponse {
    let email = payload.email.trim().to_lowercase();
    if !email.contains('@') {
        return oauth_error(StatusCode::BAD_REQUEST, "invalid_request", "邮箱格式无效.");
    }

    let code = random_numeric_code();
    let Ok(code_hash) = hash_password(&code) else {
        return oauth_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "server_error",
            "验证码生成失败.",
        );
    };
    let key = format!("oauth:email_verify:code:{email}");
    if valkey_set_ex(&state.valkey, key, code_hash, EMAIL_CODE_TTL_SECONDS)
        .await
        .is_err()
    {
        return oauth_error(
            StatusCode::SERVICE_UNAVAILABLE,
            "server_error",
            "验证码生成失败.",
        );
    }

    let mut body = json!({"success": true, "message": "如果邮箱尚未注册，验证码将会发送。"});
    if cfg!(debug_assertions) && state.settings.email_code_dev_response_enabled {
        body["verification_code"] = json!(code);
    }
    json_response(body)
}

fn random_numeric_code() -> String {
    let value = u32::from_be_bytes(rand::random::<[u8; 4]>()) % 1_000_000;
    format!("{value:06}")
}
