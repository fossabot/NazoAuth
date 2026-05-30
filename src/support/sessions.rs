//! 会话用户与权限解析。
// 只处理从请求 Cookie 到当前用户/管理员身份的解析。

use super::prelude::*;

pub(crate) async fn current_user(state: &AppState, req: &HttpRequest) -> Option<UserRow> {
    let sid = cookie_value(req, &state.settings.session_cookie_name)?;
    let user_id = valkey_get(&state.valkey, format!("oauth:session:{sid}"))
        .await
        .ok()?;
    let id = Uuid::parse_str(user_id.as_deref()?).ok()?;
    find_user_by_id(&state.diesel_db, id)
        .await
        .ok()
        .flatten()
        .filter(|u| u.is_active)
}

pub(crate) async fn require_admin(state: &AppState, req: &HttpRequest) -> Option<UserRow> {
    current_user(state, req)
        .await
        .filter(|u| u.role == "admin" && u.admin_level > 0)
}
