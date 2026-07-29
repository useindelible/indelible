use sha2::{Digest, Sha256};

pub fn obsidian_content_hash(content: &str) -> String {
    sha256_hex(content.as_bytes())
}

pub(super) fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    hex::encode(digest)
}
