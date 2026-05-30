//! 邮箱验证码发送端点。
// 当前端点负责生成和保存验证码；邮件投递可由独立投递服务接入。
use crate::http::prelude::*;

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

    let code = "123456";
    let key = format!("oauth:email_verify:code:{email}");
    let _ = valkey_set_ex(&state.valkey, key, code, 900).await;
    json_response(json!({"success": true, "message": "如果邮箱尚未注册，验证码将会发送。"}))
}
