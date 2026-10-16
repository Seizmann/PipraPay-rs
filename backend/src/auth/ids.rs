use rand::RngCore;
use sha2::{Digest, Sha256};

/// Generate a 32-byte random session token, hex-encoded (PHP: bin2hex(random_bytes(16)),
/// doubled length for headroom). The raw token goes to the cookie; only its SHA-256
/// hash is stored in `sessions.token_hash`.
pub fn generate_session_token() -> String {
    let mut buf = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut buf);
    hex(&buf)
}

pub fn hash_token(token: &str) -> String {
    let digest = Sha256::digest(token.as_bytes());
    hex(&digest)
}

pub fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// Random pairing OTP / device token (PHP generateItemID: 10-digit numeric).
pub fn generate_item_id() -> String {
    // 10-digit numeric, first digit nonzero, like PHP's implementation.
    let mut buf = [0u8; 8];
    rand::thread_rng().fill_bytes(&mut buf);
    let n = u64::from_be_bytes(buf) % 9_000_000_000 + 1_000_000_000;
    n.to_string()
}

/// PHP generateItemID(27, 27): 27-char alphanumeric ref for transactions.
pub fn generate_ref() -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let mut buf = [0u8; 27];
    rand::thread_rng().fill_bytes(&mut buf);
    buf.iter()
        .map(|b| CHARS[(*b as usize) % CHARS.len()] as char)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_token_is_hex_64() {
        let t = generate_session_token();
        assert_eq!(t.len(), 64);
        assert!(t.chars().all(|c| c.is_ascii_hexdigit()));
        assert_ne!(t, generate_session_token()); // random
    }

    #[test]
    fn token_hash_is_stable_sha256() {
        let h1 = hash_token("abc");
        let h2 = hash_token("abc");
        assert_eq!(h1, h2);
        assert_eq!(h1.len(), 64);
        assert_ne!(hash_token("abc"), hash_token("abd"));
    }

    #[test]
    fn item_id_is_10_digit_numeric() {
        for _ in 0..20 {
            let id = generate_item_id();
            assert_eq!(id.len(), 10, "{id}");
            assert!(id.chars().all(|c| c.is_ascii_digit()), "{id}");
            assert_ne!(&id[..1], "0");
        }
    }

    #[test]
    fn ref_is_27_alphanumeric() {
        for _ in 0..20 {
            let r = generate_ref();
            assert_eq!(r.len(), 27, "{r}");
            assert!(r.chars().all(|c| c.is_ascii_alphanumeric()));
        }
    }
}
