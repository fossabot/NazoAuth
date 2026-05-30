//! 管理端用户账户接口。
// 只处理用户列表与用户状态更新，不包含客户端或授权关系逻辑。
use crate::http::prelude::*;

pub(crate) async fn admin_users(
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
    let (total, user_rows) = match get_conn(&state.diesel_db).await {
        Ok(mut conn) => {
            let total = users::table
                .select(count_star())
                .first::<i64>(&mut conn)
                .await
                .unwrap_or(0);
            let rows = users::table
                .select(UserRow::as_select())
                .order(users::created_at.desc())
                .limit(page_size as i64)
                .offset(offset as i64)
                .load::<UserRow>(&mut conn)
                .await
                .unwrap_or_default();
            (total, rows)
        }
        Err(_) => (0, Vec::new()),
    };
    let items: Vec<Value> = user_rows.into_iter().map(admin_user_json).collect();
    json_response(json!({"total": total, "page": page, "page_size": page_size, "items": items}))
}

#[derive(Deserialize)]
pub(crate) struct PatchUserRequest {
    role: Option<String>,
    admin_level: Option<i32>,
    is_active: Option<bool>,
}

pub(crate) async fn admin_patch_user(
    state: Data<AppState>,
    req: HttpRequest,
    path: actix_web::web::Path<Uuid>,
    Json(payload): Json<PatchUserRequest>,
) -> HttpResponse {
    let user_id = path.into_inner();
    if !has_valid_csrf_token(&state, &req, None) {
        return csrf_error();
    }
    if require_admin(&state, &req).await.is_none() {
        return oauth_error(
            StatusCode::FORBIDDEN,
            "access_denied",
            "当前账号无管理权限.",
        );
    }
    if payload.admin_level.is_some_and(|level| level < 0) {
        return oauth_error(
            StatusCode::BAD_REQUEST,
            "invalid_request",
            "admin_level 不能为负数.",
        );
    }
    let updated = match get_conn(&state.diesel_db).await {
        Ok(mut conn) => {
            if let Some(role) = payload.role.as_deref() {
                match role {
                    "admin" | "user" => {}
                    _ => {
                        return oauth_error(
                            StatusCode::BAD_REQUEST,
                            "invalid_request",
                            "用户角色无效.",
                        );
                    }
                };
                if diesel::update(users::table.find(user_id))
                    .set((users::role.eq(role), users::updated_at.eq(diesel_now)))
                    .execute(&mut conn)
                    .await
                    .is_err()
                {
                    return oauth_error(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "server_error",
                        "用户角色更新失败.",
                    );
                }
            }
            if let Some(admin_level) = payload.admin_level
                && diesel::update(users::table.find(user_id))
                    .set((
                        users::admin_level.eq(admin_level),
                        users::updated_at.eq(diesel_now),
                    ))
                    .execute(&mut conn)
                    .await
                    .is_err()
            {
                return oauth_error(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "server_error",
                    "管理员等级更新失败.",
                );
            }
            if let Some(is_active) = payload.is_active
                && diesel::update(users::table.find(user_id))
                    .set((
                        users::is_active.eq(is_active),
                        users::updated_at.eq(diesel_now),
                    ))
                    .execute(&mut conn)
                    .await
                    .is_err()
            {
                return oauth_error(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "server_error",
                    "用户状态更新失败.",
                );
            }
            users::table
                .find(user_id)
                .select(UserRow::as_select())
                .first::<UserRow>(&mut conn)
                .await
                .optional()
                .ok()
                .flatten()
        }
        Err(_) => None,
    };
    match updated {
        Some(user) => json_response(admin_user_json(user)),
        None => oauth_error(StatusCode::NOT_FOUND, "invalid_request", "未找到该用户."),
    }
}
