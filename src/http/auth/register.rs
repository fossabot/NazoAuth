//! 用户注册端点。
// 注册只接受已验证邮箱，密码进入数据库前必须完成 Argon2 哈希。
use crate::http::prelude::*;

#[derive(Deserialize)]
pub(crate) struct RegisterRequest {
    email: String,
    verification_code: String,
    password: String,
}

/// 使用邮箱验证码创建本地用户。
pub(crate) async fn register(
    state: Data<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> HttpResponse {
    let email = payload.email.trim().to_lowercase();
    let key = format!("oauth:email_verify:code:{email}");
    let stored = valkey_get(&state.valkey, &key).await.unwrap_or(None);
    if stored.as_deref() != Some(payload.verification_code.as_str()) {
        return oauth_error(
            StatusCode::BAD_REQUEST,
            "invalid_grant",
            "验证码错误或已过期.",
        );
    }

    let _ = valkey_del(&state.valkey, &key).await;
    if find_user_by_email(&state.diesel_db, &email)
        .await
        .ok()
        .flatten()
        .is_some()
    {
        return oauth_error(StatusCode::CONFLICT, "invalid_request", "该邮箱已注册.");
    }

    let password_hash = match hash_password(&payload.password) {
        Ok(v) => v,
        Err(_) => {
            return oauth_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "server_error",
                "密码哈希失败.",
            );
        }
    };
    let username = format!("user_{}", Uuid::now_v7());
    let mut conn = match get_conn(&state.diesel_db).await {
        Ok(conn) => conn,
        Err(_) => {
            return oauth_error(
                StatusCode::SERVICE_UNAVAILABLE,
                "server_error",
                "数据库连接失败.",
            );
        }
    };

    let row = diesel::insert_into(users::table)
        .values((
            users::username.eq(username),
            users::email.eq(email),
            users::password_hash.eq(password_hash),
            users::email_verified.eq(true),
        ))
        .returning(UserRow::as_returning())
        .get_result::<UserRow>(&mut conn)
        .await;
    match row {
        Ok(user) => json_response_status(
            StatusCode::CREATED,
            json!({"id": user.id, "email": user.email}),
        ),
        Err(_) => oauth_error(StatusCode::CONFLICT, "invalid_request", "该邮箱已注册."),
    }
}
