//! 当前用户已授权应用接口。
// 只读取当前用户的 OAuth 授权关系。
use crate::http::prelude::*;

pub(crate) async fn my_applications(state: Data<AppState>, req: HttpRequest) -> HttpResponse {
    let Some(user) = current_user(&state, &req).await else {
        return oauth_error(
            StatusCode::UNAUTHORIZED,
            "login_required",
            "会话不存在或已过期,请重新登录.",
        );
    };
    let rows = match get_conn(&state.diesel_db).await {
        Ok(mut conn) => user_client_grants::table
            .inner_join(
                oauth_clients::table.on(oauth_clients::id.eq(user_client_grants::client_id)),
            )
            .filter(user_client_grants::user_id.eq(user.id))
            .select((
                oauth_clients::client_id,
                oauth_clients::client_name,
                user_client_grants::last_scopes,
                user_client_grants::last_authorized_at,
                user_client_grants::authorization_count,
            ))
            .order(user_client_grants::last_authorized_at.desc())
            .load::<MyApplicationRow>(&mut conn)
            .await
            .unwrap_or_default(),
        Err(_) => Vec::new(),
    };
    let items: Vec<Value> = rows
        .into_iter()
        .map(|r| json!({"client_id": r.client_id, "client_name": r.client_name, "last_scopes": json_array_to_strings(&r.last_scopes), "last_authorized_at": r.last_authorized_at, "authorization_count": r.authorization_count}))
        .collect();
    json_response(json!({"total": items.len(), "items": items}))
}
