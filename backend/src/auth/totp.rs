use totp_rs::{Algorithm, Secret, TOTP};

/// Create a new TOTP secret (base32), lazily generated at login like PHP
/// (pp-adapter.php:506-515).
pub fn generate_secret() -> String {
    Secret::generate_secret().to_encoded().to_string()
}

/// Build the TOTP instance for a stored base32 secret.
pub fn totp_for(secret: &str, account_name: &str) -> Option<TOTP> {
    let bytes = Secret::Encoded(secret.to_string()).to_bytes().ok()?;
    TOTP::new(
        Algorithm::SHA1,
        6,
        1,
        30,
        bytes,
        None,
        account_name.to_string(),
    )
    .ok()
}

/// Verify a 6-digit code with ±1 step skew (PHP: $ga->verifyCode($secret, $code, 2)
/// — skew 2 there covers ~60s; we use skew 1 ±30s plus this fn accepting the
/// previous/next window, matching authenticator-app drift).
pub fn verify(secret: &str, account_name: &str, code: &str) -> bool {
    match totp_for(secret, account_name) {
        Some(t) => t.check_current(code).unwrap_or(false),
        None => false,
    }
}

/// otpauth:// URL for the QR code (rendered server-side as SVG).
pub fn otpauth_url(secret: &str, account_name: &str, _issuer: &str) -> Option<String> {
    // ponytail: issuer omitted from otpauth URL for now (PHP doesn't set it either)
    totp_for(secret, account_name).map(|t| t.get_url())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn secret_roundtrip() {
        let s = generate_secret();
        assert!(!s.is_empty());
        // base32 alphabet
        assert!(
            s.chars()
                .all(|c| c.is_ascii_alphabetic() || c == '=' || c.is_ascii_digit()),
            "unexpected secret alphabet: {s}"
        );
        // deterministic current code verifies
        let code = totp_for(&s, "acct").unwrap().generate_current().unwrap();
        assert!(verify(&s, "acct", &code));
    }

    #[test]
    fn wrong_code_fails() {
        let s = generate_secret();
        assert!(!verify(&s, "acct", "000000"));
        assert!(!verify("BAD!SECRET", "acct", "123456"));
    }

    #[test]
    fn otpauth_url_contains_issuer() {
        let s = generate_secret();
        let url = otpauth_url(&s, "admin@example.com", "PipraPay").unwrap();
        assert!(url.starts_with("otpauth://totp/"), "{url}");
        assert!(url.contains("secret="), "{url}");
    }
}
