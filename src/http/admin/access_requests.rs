//! 管理端客户端接入申请接口。
// 申请审批会创建客户端，因此显式依赖 clients 模块的创建逻辑。
use super::clients::{CreateClientRequest, insert_client_row};
use crate::http::prelude::*;

pub(crate) async fn admin_access_requests(
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
    let status = match q
        .get("status")
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
    {
        Some(value) => match value
            .parse::<i16>()
            .ok()
            .and_then(AccessRequestStatus::from_code)
        {
            Some(status) => Some(status),
            None => {
                return oauth_error(
                    StatusCode::BAD_REQUEST,
                    "invalid_request",
                    "status 参数仅支持 0/1/2.",
                );
            }
        },
        None => None,
    };
    let search = q.get("q").map(String::as_str);
    let total = access_request_count(&state.diesel_db, search, status).await;
    let rows = access_request_rows(&state.diesel_db, page_size, offset, search, status).await;
    json_response(json!({"total": total, "page": page, "page_size": page_size, "items": rows}))
}

pub(crate) async fn admin_approve_access_request(
    state: Data<AppState>,
    req: HttpRequest,
    path: actix_web::web::Path<Uuid>,
    Json(payload): Json<CreateClientRequest>,
) -> HttpResponse {
    let request_id = path.into_inner();
    if !has_valid_csrf_token(&state, &req, None) {
        return csrf_error();
    }
    let Some(admin) = require_admin(&state, &req).await else {
        return oauth_error(
            StatusCode::FORBIDDEN,
            "access_denied",
            "当前账号无管理权限.",
        );
    };
    let Some(pending_request) = (match get_conn(&state.diesel_db).await {
        Ok(mut conn) => client_access_requests::table
            .filter(client_access_requests::id.eq(request_id))
            .filter(client_access_requests::status.eq(AccessRequestStatus::Pending.code()))
            .select((
                client_access_requests::user_id,
                client_access_requests::site_name,
            ))
            .first::<PendingAccessRequestRow>(&mut conn)
            .await
            .optional()
            .ok()
            .flatten(),
        Err(_) => None,
    }) else {
        return oauth_error(
            StatusCode::CONFLICT,
            "invalid_request",
            "该申请已处理,不可重复审批.",
        );
    };
    let request_user_id = pending_request.user_id;
    let site_name = pending_request.site_name;
    match insert_client_row(&state, payload).await {
        Ok((client, issued_secret)) => {
            if let Ok(mut conn) = get_conn(&state.diesel_db).await {
                let _ = diesel::update(client_access_requests::table.find(request_id))
                    .set((
                        client_access_requests::status.eq(AccessRequestStatus::Approved.code()),
                        client_access_requests::resolved_by_user_id.eq(admin.id),
                        client_access_requests::approved_client_id.eq(client.id),
                        client_access_requests::resolved_at.eq(diesel_now),
                        client_access_requests::updated_at.eq(diesel_now),
                    ))
                    .execute(&mut conn)
                    .await;
            }
            let token = Uuid::now_v7().to_string();
            let expires_at =
                Utc::now() + Duration::seconds(state.settings.client_delivery_ttl_seconds as i64);
            let payload = json!({
                "request_id": request_id,
                "user_id": request_user_id,
                "client_id": client.client_id,
                "client_name": client.client_name,
                "client_type": client.client_type,
                "client_secret": issued_secret,
                "redirect_uris": json_array_to_strings(&client.redirect_uris),
                "scopes": json_array_to_strings(&client.scopes),
                "grant_types": json_array_to_strings(&client.grant_types),
                "token_endpoint_auth_method": client.token_endpoint_auth_method,
                "site_name": site_name,
                "created_at": Utc::now(),
                "expires_at": expires_at
            });
            let _ = valkey_set_ex(
                &state.valkey,
                format!("oauth:client_delivery:{request_user_id}:{token}"),
                payload.to_string(),
                state.settings.client_delivery_ttl_seconds,
            )
            .await;
        }
        Err(e) => {
            return oauth_error(
                StatusCode::BAD_REQUEST,
                "invalid_request",
                &format!("客户端创建失败: {e}"),
            );
        }
    }
    let rows = access_request_by_id(&state.diesel_db, request_id).await;
    json_response(rows.unwrap_or_else(|| json!({"id": request_id})))
}

#[derive(Deserialize)]
pub(crate) struct RejectAccessRequest {
    admin_note: String,
}

pub(crate) async fn admin_reject_access_request(
    state: Data<AppState>,
    req: HttpRequest,
    path: actix_web::web::Path<Uuid>,
    Json(payload): Json<RejectAccessRequest>,
) -> HttpResponse {
    let request_id = path.into_inner();
    if !has_valid_csrf_token(&state, &req, None) {
        return csrf_error();
    }
    let Some(admin) = require_admin(&state, &req).await else {
        return oauth_error(
            StatusCode::FORBIDDEN,
            "access_denied",
            "当前账号无管理权限.",
        );
    };
    let updated = match get_conn(&state.diesel_db).await {
        Ok(mut conn) => diesel::update(
            client_access_requests::table
                .find(request_id)
                .filter(client_access_requests::status.eq(AccessRequestStatus::Pending.code())),
        )
        .set((
            client_access_requests::status.eq(AccessRequestStatus::Rejected.code()),
            client_access_requests::admin_note.eq(payload.admin_note),
            client_access_requests::resolved_by_user_id.eq(admin.id),
            client_access_requests::resolved_at.eq(diesel_now),
            client_access_requests::updated_at.eq(diesel_now),
        ))
        .execute(&mut conn)
        .await
        .unwrap_or(0),
        Err(_) => 0,
    };
    if updated == 0 {
        return oauth_error(
            StatusCode::CONFLICT,
            "invalid_request",
            "该申请已处理,不可重复拒绝.",
        );
    }
    json_response(
        access_request_by_id(&state.diesel_db, request_id)
            .await
            .unwrap_or_else(|| json!({"id": request_id})),
    )
}
