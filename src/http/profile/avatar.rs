//! 当前用户头像接口。
// 只处理头像上传、读取和删除的 HTTP 细节。
use crate::http::prelude::*;

pub(crate) async fn upload_avatar(
    state: Data<AppState>,
    req: HttpRequest,
    mut multipart: Multipart,
) -> HttpResponse {
    if !has_valid_csrf_token(&state, &req, None) {
        return csrf_error();
    }
    let Some(user) = current_user(&state, &req).await else {
        return login_required_response(&state);
    };
    while let Some(field) = multipart.next().await {
        let mut field = match field {
            Ok(field) => field,
            Err(_) => {
                return oauth_error(
                    StatusCode::BAD_REQUEST,
                    "invalid_request",
                    "头像文件读取失败.",
                );
            }
        };
        if field.name() != Some("avatar") {
            continue;
        }
        let mut bytes = Vec::new();
        while let Some(chunk) = field.next().await {
            let chunk = match chunk {
                Ok(chunk) => chunk,
                Err(_) => {
                    return oauth_error(
                        StatusCode::BAD_REQUEST,
                        "invalid_request",
                        "头像文件读取失败.",
                    );
                }
            };
            bytes.extend_from_slice(&chunk);
            if bytes.len() > state.settings.avatar_max_bytes {
                return oauth_error(
                    StatusCode::PAYLOAD_TOO_LARGE,
                    "invalid_request",
                    "头像文件过大.",
                );
            }
        }
        let Some(content_type) = detect_avatar_content_type(&bytes) else {
            return oauth_error(
                StatusCode::BAD_REQUEST,
                "invalid_request",
                "头像仅支持 PNG、JPEG、WEBP 格式.",
            );
        };
        let version = Uuid::now_v7().to_string();
        let user_dir = avatar_user_dir(&state, user.id);
        if tokio::fs::create_dir_all(&user_dir).await.is_err() {
            return oauth_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "server_error",
                "头像保存失败.",
            );
        }
        if tokio::fs::write(avatar_path(&state, user.id), &bytes)
            .await
            .is_err()
            || tokio::fs::write(
                avatar_meta_path(&state, user.id),
                json!({"content_type": content_type, "version": version}).to_string(),
            )
            .await
            .is_err()
        {
            return oauth_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "server_error",
                "头像保存失败.",
            );
        }
        if let Ok(mut conn) = get_conn(&state.diesel_db).await {
            let _ = diesel::update(users::table.find(user.id))
                .set((
                    users::avatar_url.eq(Some(format!("/auth/me/avatar?v={version}"))),
                    users::updated_at.eq(diesel_now),
                ))
                .execute(&mut conn)
                .await;
        }
        return json_response(auth_me_json(&state, &user).await);
    }
    oauth_error(
        StatusCode::BAD_REQUEST,
        "invalid_request",
        "缺少 avatar 文件.",
    )
}

pub(crate) async fn get_avatar(state: Data<AppState>, req: HttpRequest) -> HttpResponse {
    let Some(user) = current_user(&state, &req).await else {
        return login_required_response(&state);
    };
    if is_cross_site_fetch(req.headers()) {
        return oauth_error(
            StatusCode::FORBIDDEN,
            "access_denied",
            "跨站请求头像资源被拒绝.",
        );
    };
    match tokio::fs::read(avatar_path(&state, user.id)).await {
        Ok(bytes) => {
            let mut resp = bytes_response(bytes);
            let content_type = read_avatar_content_type(&state, user.id)
                .await
                .unwrap_or("image/png");
            if let Ok(value) = HeaderValue::from_str(content_type) {
                resp.headers_mut().insert(header::CONTENT_TYPE, value);
            }
            resp.headers_mut().insert(
                header::CACHE_CONTROL,
                HeaderValue::from_static("private, no-store, no-cache, must-revalidate"),
            );
            resp.headers_mut()
                .insert(header::PRAGMA, HeaderValue::from_static("no-cache"));
            resp.headers_mut().insert(
                header::X_CONTENT_TYPE_OPTIONS,
                HeaderValue::from_static("nosniff"),
            );
            resp.headers_mut().insert(
                header::CONTENT_SECURITY_POLICY,
                HeaderValue::from_static("default-src 'none'"),
            );
            resp
        }
        Err(_) => oauth_error(
            StatusCode::NOT_FOUND,
            "invalid_request",
            "当前用户尚未上传头像.",
        ),
    }
}

pub(crate) async fn delete_avatar(state: Data<AppState>, req: HttpRequest) -> HttpResponse {
    if !has_valid_csrf_token(&state, &req, None) {
        return csrf_error();
    }
    let Some(user) = current_user(&state, &req).await else {
        return login_required_response(&state);
    };
    let _ = tokio::fs::remove_file(avatar_path(&state, user.id)).await;
    let _ = tokio::fs::remove_file(avatar_meta_path(&state, user.id)).await;
    let _ = tokio::fs::remove_dir(avatar_user_dir(&state, user.id)).await;
    if let Ok(mut conn) = get_conn(&state.diesel_db).await {
        let _ = diesel::update(users::table.find(user.id))
            .set((
                users::avatar_url.eq(Option::<String>::None),
                users::updated_at.eq(diesel_now),
            ))
            .execute(&mut conn)
            .await;
    }
    json_response(auth_me_json(&state, &user).await)
}
