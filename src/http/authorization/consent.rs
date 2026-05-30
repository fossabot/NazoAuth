//! 授权确认页数据端点。
// 前端通过 request_id 读取待确认内容，服务端再次校验该请求属于当前用户。
use crate::http::prelude::*;

/// 返回授权确认页所需的客户端、scope 和 CSRF 信息。
pub(crate) async fn authorize_consent(
    state: Data<AppState>,
    req: HttpRequest,
    Query(q): Query<HashMap<String, String>>,
) -> HttpResponse {
    let Some(user) = current_user(&state, &req).await else {
        return oauth_error(
            StatusCode::UNAUTHORIZED,
            "login_required",
            "授权前必须先登录.",
        );
    };
    let Some(request_id) = q.get("request_id") else {
        return oauth_error(
            StatusCode::BAD_REQUEST,
            "invalid_request",
            "缺少 request_id.",
        );
    };

    let raw = valkey_get(&state.valkey, format!("oauth:consent:{request_id}"))
        .await
        .unwrap_or(None);
    let Some(payload) = raw.and_then(|v| serde_json::from_str::<ConsentPayload>(&v).ok()) else {
        return oauth_error(
            StatusCode::BAD_REQUEST,
            "invalid_request",
            "授权请求不存在或已过期,请重新发起授权.",
        );
    };
    if payload.user_id != user.id {
        return oauth_error(
            StatusCode::FORBIDDEN,
            "access_denied",
            "当前会话与授权请求不匹配.",
        );
    }

    json_response(json!({
        "request_id": payload.request_id,
        "client_id": payload.client_id,
        "client_name": payload.client_name,
        "redirect_uri": payload.redirect_uri,
        "scopes": payload.scopes,
        "csrf_token": cookie_value(&req, &state.settings.csrf_cookie_name)
    }))
}
