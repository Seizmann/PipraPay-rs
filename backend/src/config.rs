// ponytail: config parsed from SECRETS.md-style env file; single flat struct.
use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub session_secret: String,
    pub public_base_url: String,
    pub setup_token: String,
    pub smtp_host: Option<String>,
    pub smtp_port: u16,
    pub smtp_user: Option<String>,
    pub smtp_pass: Option<String>,
    pub smtp_from: Option<String>,
    pub telegram_bot_token: Option<String>,
}

/// Secrets may live in env vars or in a `KEY=value` file (SECRETS.md fenced
/// blocks are stripped of the ``` lines). Env wins.
pub fn load(secrets_path: &str) -> anyhow::Result<Config> {
    let mut map: std::collections::HashMap<String, String> = std::collections::HashMap::new();

    if let Ok(content) = std::fs::read_to_string(secrets_path) {
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("```") || line.starts_with('#') || line.is_empty() {
                continue;
            }
            if let Some((k, v)) = line.split_once('=') {
                map.entry(k.trim().to_string())
                    .or_insert_with(|| v.trim().trim_matches('"').to_string());
            }
        }
    }

    let get = |key: &str| -> Option<String> {
        env::var(key)
            .ok()
            .filter(|v| !v.is_empty())
            .or_else(|| map.get(key).cloned().filter(|v| !v.is_empty()))
    };

    let missing = |key: &str| -> anyhow::Error {
        anyhow::anyhow!(
            "Missing required secret `{key}`. Add it to {secrets_path} or the environment.",
            secrets_path = secrets_path
        )
    };

    let cfg = Config {
        database_url: get("DATABASE_URL").ok_or_else(|| missing("DATABASE_URL"))?,
        session_secret: get("SESSION_SECRET").ok_or_else(|| missing("SESSION_SECRET"))?,
        public_base_url: get("PUBLIC_BASE_URL").ok_or_else(|| missing("PUBLIC_BASE_URL"))?,
        setup_token: get("SETUP_TOKEN").ok_or_else(|| missing("SETUP_TOKEN"))?,
        smtp_host: get("SMTP_HOST"),
        smtp_port: get("SMTP_PORT").and_then(|v| v.parse().ok()).unwrap_or(587),
        smtp_user: get("SMTP_USER"),
        smtp_pass: get("SMTP_PASS"),
        smtp_from: get("SMTP_FROM"),
        telegram_bot_token: get("TELEGRAM_BOT_TOKEN"),
    };

    if !cfg.public_base_url.starts_with("http://") && !cfg.public_base_url.starts_with("https://") {
        anyhow::bail!("PUBLIC_BASE_URL must start with http:// or https://");
    }

    Ok(cfg)
}

impl Config {
    pub fn cookie_secure(&self) -> bool {
        self.public_base_url.starts_with("https://")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cookie_secure_iff_https() {
        let base = |url: &str| Config {
            database_url: "x".into(),
            session_secret: "x".into(),
            public_base_url: url.into(),
            setup_token: "x".into(),
            smtp_host: None,
            smtp_port: 587,
            smtp_user: None,
            smtp_pass: None,
            smtp_from: None,
            telegram_bot_token: None,
        };
        assert!(base("https://pay.example.com").cookie_secure());
        assert!(!base("http://localhost:8080").cookie_secure());
    }

    #[test]
    fn load_fails_when_missing() {
        // No env, no file at a bogus path -> must fail loudly.
        let err = load("/nonexistent/secrets.md");
        assert!(err.is_err());
    }

    #[test]
    fn load_parses_key_value_file() {
        let dir = std::env::temp_dir().join(format!("pp-cfg-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("secrets.md");
        std::fs::write(
            &path,
            "# comment\n```\nDATABASE_URL=postgres://db\nSESSION_SECRET=s3cr\nPUBLIC_BASE_URL=https://pay.test\nSETUP_TOKEN=tok\n```\n",
        )
        .unwrap();
        let cfg = load(path.to_str().unwrap()).unwrap();
        assert_eq!(cfg.database_url, "postgres://db");
        assert_eq!(cfg.session_secret, "s3cr");
        assert_eq!(cfg.public_base_url, "https://pay.test");
        assert_eq!(cfg.setup_token, "tok");
        std::fs::remove_dir_all(&dir).ok();
    }
}
