//! OIDC userinfo 端点。
// 只根据 Bearer access token 返回用户声明。
use crate::http::prelude::*;

pub(crate) async fn userinfo(state: Data<AppState>, req: HttpRequest) -> HttpResponse {
    let Some(token) = bearer_token(req.headers()) else {
        return oauth_bearer_error(StatusCode::UNAUTHORIZED, "invalid_token", "缺少访问令牌.");
    };
    let Some(claims) = decode_access_claims(&state, &token) else {
        return oauth_bearer_error(
            StatusCode::UNAUTHORIZED,
            "invalid_token",
            "访问令牌无效或已过期.",
        );
    };
    let preferred_username = match Uuid::parse_str(&claims.sub) {
        Ok(user_id) => find_user_by_id(&state.diesel_db, user_id)
            .await
            .ok()
            .flatten()
            .map(|user| user.email),
        Err(_) => None,
    };
    json_response(json!({
        "sub": claims.sub,
        "preferred_username": preferred_username
    }))
}
