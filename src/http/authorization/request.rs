//! 授权请求入口端点。
// 该端点只创建 consent 临时状态，不签发授权码。
use crate::http::prelude::*;

/// 校验 OAuth authorize 参数并创建待确认授权请求。
pub(crate) async fn authorize(
    state: Data<AppState>,
    req: HttpRequest,
    Query(q): Query<HashMap<String, String>>,
) -> HttpResponse {
    let Some(user) = current_user(&state, &req).await else {
        let query = q
            .iter()
            .map(|(k, v)| format!("{}={}", urlencoding::encode(k), urlencoding::encode(v)))
            .collect::<Vec<_>>()
            .join("&");
        let next = if query.is_empty() {
            "/authorize".to_string()
        } else {
            format!("/authorize?{query}")
        };
        return redirect_found(format!(
            "{}/auth?next={}",
            state.settings.frontend_base_url.trim_end_matches('/'),
            urlencoding::encode(&next)
        ));
    };

    if q.get("response_type").map(String::as_str) != Some("code") {
        return oauth_error(
            StatusCode::BAD_REQUEST,
            "unsupported_response_type",
            "仅支持 response_type=code.",
        );
    }
    let Some(client_id) = q.get("client_id") else {
        return oauth_error(
            StatusCode::BAD_REQUEST,
            "invalid_request",
            "缺少 client_id.",
        );
    };
    let Some(redirect_uri) = q.get("redirect_uri") else {
        return oauth_error(
            StatusCode::BAD_REQUEST,
            "invalid_request",
            "缺少 redirect_uri.",
        );
    };
    let Some(code_challenge) = q.get("code_challenge") else {
        return oauth_error(
            StatusCode::BAD_REQUEST,
            "invalid_request",
            "缺少 code_challenge.",
        );
    };

    let Some(client) = find_client(&state.diesel_db, client_id)
        .await
        .ok()
        .flatten()
    else {
        return oauth_error(
            StatusCode::UNAUTHORIZED,
            "unauthorized_client",
            "客户端不存在或已停用.",
        );
    };
    if !client.is_active || !json_array_to_strings(&client.redirect_uris).contains(redirect_uri) {
        return oauth_error(
            StatusCode::BAD_REQUEST,
            "invalid_request",
            "redirect_uri 与客户端注册信息不匹配.",
        );
    }

    let requested_scopes = parse_scope(q.get("scope").map(String::as_str).unwrap_or(""));
    if !is_subset(&requested_scopes, &json_array_to_strings(&client.scopes)) {
        return oauth_error(
            StatusCode::BAD_REQUEST,
            "invalid_scope",
            "请求的作用域超出客户端允许范围.",
        );
    }

    let now = Utc::now();
    let request_id = Uuid::now_v7().to_string();
    let payload = ConsentPayload {
        request_id: request_id.clone(),
        user_id: user.id,
        client_id: client.client_id,
        client_name: client.client_name,
        redirect_uri: redirect_uri.clone(),
        scopes: requested_scopes,
        state: q.get("state").cloned(),
        nonce: q.get("nonce").cloned(),
        code_challenge: code_challenge.clone(),
        code_challenge_method: "S256".into(),
        issued_at: now,
        expires_at: now + Duration::seconds(state.settings.auth_code_ttl_seconds as i64),
    };
    let key = format!("oauth:consent:{request_id}");
    let body = serde_json::to_string(&payload).unwrap();
    let _ = valkey_set_ex(
        &state.valkey,
        key,
        body,
        state.settings.auth_code_ttl_seconds,
    )
    .await;

    redirect_found(format!(
        "{}/consent?request_id={request_id}",
        state.settings.frontend_base_url.trim_end_matches('/')
    ))
}
