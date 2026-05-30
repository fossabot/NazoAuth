//! Ed25519 JWK/PEM 密钥管理。
// 负责加载、生成和编码 OAuth/OIDC 签名密钥。

use super::prelude::*;

pub(crate) async fn load_or_create_keyset(settings: &Settings) -> anyhow::Result<Keyset> {
    tokio::fs::create_dir_all(&settings.jwk_keys_dir).await?;
    let keyset_path = settings.jwk_keys_dir.join("keyset.json");
    if let Some(keyset) = try_load_keyset(settings, &keyset_path).await {
        return Ok(keyset);
    }
    create_new_keyset(settings).await
}

pub(crate) async fn try_load_keyset(settings: &Settings, keyset_path: &PathBuf) -> Option<Keyset> {
    let raw = tokio::fs::read_to_string(keyset_path).await.ok()?;
    let payload = serde_json::from_str::<Value>(&raw).ok()?;
    let active_kid = payload.get("active_kid").and_then(Value::as_str)?;
    let keys = payload.get("keys").and_then(Value::as_array)?;
    let entry = keys
        .iter()
        .find(|entry| entry.get("kid").and_then(Value::as_str) == Some(active_kid))?;
    let file_name = entry.get("file").and_then(Value::as_str)?;
    let raw_key = tokio::fs::read_to_string(settings.jwk_keys_dir.join(file_name))
        .await
        .ok()?;
    let der = pem_to_der(&raw_key)?;
    keyset_from_der(active_kid, der)
}

pub(crate) async fn create_new_keyset(settings: &Settings) -> anyhow::Result<Keyset> {
    let seed: [u8; 32] = rand::random();
    let signing_key = SigningKey::from_bytes(&seed);
    let public_key = signing_key.verifying_key().to_bytes();
    let private_pkcs8_der = ed25519_pkcs8_private_der(&seed);
    let kid = format!("ed25519-{}", Uuid::now_v7());
    let file_name = format!("{kid}.pem");
    let pem = der_to_pem(&private_pkcs8_der, "PRIVATE KEY");
    tokio::fs::write(settings.jwk_keys_dir.join(&file_name), pem).await?;
    let now = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    let payload = json!({
        "active_kid": kid,
        "keys": [{
            "kid": kid,
            "file": file_name,
            "created_at": now,
            "retire_at": null
        }]
    });
    tokio::fs::write(
        settings.jwk_keys_dir.join("keyset.json"),
        serde_json::to_string_pretty(&payload)?,
    )
    .await?;
    Ok(Keyset {
        active_kid: payload["active_kid"]
            .as_str()
            .unwrap_or_default()
            .to_string(),
        private_pkcs8_der,
        public_key,
    })
}

pub(crate) fn keyset_from_der(active_kid: &str, private_pkcs8_der: Vec<u8>) -> Option<Keyset> {
    let seed = ed25519_seed_from_pkcs8(&private_pkcs8_der)?;
    let signing_key = SigningKey::from_bytes(&seed);
    Some(Keyset {
        active_kid: active_kid.to_string(),
        private_pkcs8_der,
        public_key: signing_key.verifying_key().to_bytes(),
    })
}

pub(crate) fn ed25519_pkcs8_private_der(seed: &[u8; 32]) -> Vec<u8> {
    let mut der = Vec::with_capacity(48);
    der.extend_from_slice(&[
        0x30, 0x2e, 0x02, 0x01, 0x00, 0x30, 0x05, 0x06, 0x03, 0x2b, 0x65, 0x70, 0x04, 0x22, 0x04,
        0x20,
    ]);
    der.extend_from_slice(seed);
    der
}

pub(crate) fn ed25519_seed_from_pkcs8(der: &[u8]) -> Option<[u8; 32]> {
    const PREFIX: &[u8] = &[
        0x30, 0x2e, 0x02, 0x01, 0x00, 0x30, 0x05, 0x06, 0x03, 0x2b, 0x65, 0x70, 0x04, 0x22, 0x04,
        0x20,
    ];
    if der.len() != PREFIX.len() + 32 || !der.starts_with(PREFIX) {
        return None;
    }
    let mut seed = [0u8; 32];
    seed.copy_from_slice(&der[PREFIX.len()..]);
    Some(seed)
}

pub(crate) fn der_to_pem(der: &[u8], label: &str) -> String {
    let encoded = STANDARD.encode(der);
    let mut pem = format!("-----BEGIN {label}-----\n");
    for chunk in encoded.as_bytes().chunks(64) {
        pem.push_str(std::str::from_utf8(chunk).unwrap_or_default());
        pem.push('\n');
    }
    pem.push_str(&format!("-----END {label}-----\n"));
    pem
}

pub(crate) fn pem_to_der(pem: &str) -> Option<Vec<u8>> {
    let body: String = pem
        .lines()
        .filter(|line| !line.starts_with("-----"))
        .map(str::trim)
        .collect();
    STANDARD.decode(body).ok()
}

impl Keyset {
    pub(crate) fn jwks(&self) -> Value {
        json!({
            "keys": [{
                "kty": "OKP",
                "crv": "Ed25519",
                "x": URL_SAFE_NO_PAD.encode(self.public_key),
                "use": "sig",
                "alg": "EdDSA",
                "kid": self.active_kid
            }]
        })
    }
}
