//! Ed25519 JWT 签名密钥材料。
// 私钥以 PKCS#8 DER 保存在内存中，公钥直接用于 JWKS 输出和验签。

/// 当前服务实例可用的 JWT keyset。
#[derive(Clone)]
pub(crate) struct Keyset {
    pub(crate) active_kid: String,
    pub(crate) private_pkcs8_der: Vec<u8>,
    pub(crate) public_key: [u8; 32],
}
