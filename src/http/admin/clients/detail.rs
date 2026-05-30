//! 管理端客户端详情端点。
// 根据公开 client_id 查找客户端，响应中不暴露 secret hash。
use crate::http::prelude::*;

/// 返回单个 OAuth 客户端详情。
pub(crate) async fn admin_get_client(
    state: Data<AppState>,
    req: HttpRequest,
    path: actix_web::web::Path<String>,
) -> HttpResponse {
    let client_id = path.into_inner();
    if require_admin(&state, &req).await.is_none() {
        return oauth_error(
            StatusCode::FORBIDDEN,
            "access_denied",
            "当前账号无管理权限.",
        );
    }

    match find_client(&state.diesel_db, &client_id)
        .await
        .ok()
        .flatten()
    {
        Some(client) => json_response(client_json(client)),
        None => oauth_error(StatusCode::NOT_FOUND, "invalid_request", "未找到该客户端."),
    }
}
