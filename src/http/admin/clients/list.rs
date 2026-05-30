//! 管理端客户端列表端点。
// 只负责分页读取和响应组装，不处理创建或更新逻辑。
use crate::http::prelude::*;

/// 返回 OAuth 客户端分页列表。
pub(crate) async fn admin_clients(
    state: Data<AppState>,
    req: HttpRequest,
    Query(q): Query<HashMap<String, String>>,
) -> HttpResponse {
    if require_admin(&state, &req).await.is_none() {
        return oauth_error(
            StatusCode::FORBIDDEN,
            "access_denied",
            "当前账号无管理权限.",
        );
    }

    let (page, page_size, offset) = pagination(&q);
    let (total, clients) = match get_conn(&state.diesel_db).await {
        Ok(mut conn) => {
            let total = oauth_clients::table
                .select(count_star())
                .first::<i64>(&mut conn)
                .await
                .unwrap_or(0);
            let rows = oauth_clients::table
                .select(ClientRow::as_select())
                .order(oauth_clients::created_at.desc())
                .limit(page_size as i64)
                .offset(offset as i64)
                .load::<ClientRow>(&mut conn)
                .await
                .unwrap_or_default();
            (total, rows)
        }
        Err(_) => (0, Vec::new()),
    };
    let items: Vec<Value> = clients.into_iter().map(client_json).collect();
    json_response(json!({"total": total, "page": page, "page_size": page_size, "items": items}))
}
