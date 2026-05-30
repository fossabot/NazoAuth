//! OAuth 作用域、audience 与授权关系工具。
// 只处理 OAuth 语义中的集合判断和授权记录 upsert。

use super::prelude::*;

pub(crate) fn json_array_to_strings(value: &Value) -> Vec<String> {
    value
        .as_array()
        .map(|items| {
            items
                .iter()
                .filter_map(|v| v.as_str().map(ToOwned::to_owned))
                .collect()
        })
        .unwrap_or_default()
}

pub(crate) fn parse_scope(raw: &str) -> Vec<String> {
    raw.split_whitespace()
        .map(ToOwned::to_owned)
        .filter(|v| !v.is_empty())
        .collect()
}

pub(crate) fn is_subset(requested: &[String], allowed: &[String]) -> bool {
    requested.iter().all(|s| allowed.contains(s))
}

pub(crate) fn audience_allowed(client: &ClientRow, audience: &str) -> bool {
    json_array_to_strings(&client.allowed_audiences)
        .iter()
        .any(|allowed| allowed == audience)
}

pub(crate) fn sorted_scope_string(scopes: &[String]) -> String {
    let mut values = scopes.to_vec();
    values.sort();
    values.dedup();
    values.join(" ")
}

pub(crate) async fn upsert_grant(
    state: &AppState,
    user_id: Uuid,
    client_id: &str,
    scopes: &[String],
) -> anyhow::Result<()> {
    let Some(client) = find_client(&state.diesel_db, client_id)
        .await
        .ok()
        .flatten()
    else {
        return Ok(());
    };
    let now = Utc::now();
    let mut conn = get_conn(&state.diesel_db).await?;
    diesel::insert_into(user_client_grants::table)
        .values((
            user_client_grants::user_id.eq(user_id),
            user_client_grants::client_id.eq(client.id),
            user_client_grants::first_authorized_at.eq(now),
            user_client_grants::last_authorized_at.eq(now),
            user_client_grants::last_scopes.eq(json!(scopes)),
            user_client_grants::authorization_count.eq(1),
        ))
        .on_conflict((user_client_grants::user_id, user_client_grants::client_id))
        .do_update()
        .set((
            user_client_grants::last_authorized_at.eq(now),
            user_client_grants::last_scopes.eq(json!(scopes)),
            user_client_grants::authorization_count.eq(user_client_grants::authorization_count + 1),
        ))
        .execute(&mut conn)
        .await?;
    Ok(())
}
