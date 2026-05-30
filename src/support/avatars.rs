//! 头像文件存储辅助函数。
// 只处理头像路径、类型识别和元数据读取。

use super::prelude::*;

pub(crate) fn avatar_path(state: &AppState, user_id: Uuid) -> PathBuf {
    avatar_user_dir(state, user_id).join("avatar.bin")
}

pub(crate) fn avatar_meta_path(state: &AppState, user_id: Uuid) -> PathBuf {
    avatar_user_dir(state, user_id).join("meta.json")
}

pub(crate) fn avatar_user_dir(state: &AppState, user_id: Uuid) -> PathBuf {
    state.settings.avatar_storage_dir.join(user_id.to_string())
}

pub(crate) fn detect_avatar_content_type(bytes: &[u8]) -> Option<&'static str> {
    if bytes.is_empty() {
        return None;
    }
    if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        return Some("image/png");
    }
    if bytes.starts_with(b"\xff\xd8\xff") {
        return Some("image/jpeg");
    }
    if bytes.len() >= 12 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        return Some("image/webp");
    }
    None
}

pub(crate) async fn read_avatar_meta(state: &AppState, user_id: Uuid) -> Option<Value> {
    let raw = tokio::fs::read_to_string(avatar_meta_path(state, user_id))
        .await
        .ok()?;
    serde_json::from_str(&raw).ok()
}

pub(crate) async fn read_avatar_content_type(
    state: &AppState,
    user_id: Uuid,
) -> Option<&'static str> {
    match read_avatar_meta(state, user_id)
        .await?
        .get("content_type")?
        .as_str()?
    {
        "image/png" => Some("image/png"),
        "image/jpeg" => Some("image/jpeg"),
        "image/webp" => Some("image/webp"),
        _ => None,
    }
}

pub(crate) async fn read_avatar_version(state: &AppState, user_id: Uuid) -> Option<String> {
    if !tokio::fs::try_exists(avatar_path(state, user_id))
        .await
        .ok()?
    {
        return None;
    }
    read_avatar_meta(state, user_id)
        .await?
        .get("version")?
        .as_str()
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}
