use std::net::SocketAddr;

use reqwest::Url;

pub const DEFAULT_BIND_ADDR: &str = "127.0.0.1:3000";
pub const DEFAULT_POLL_INTERVAL_SECONDS: u64 = 15;

#[derive(Clone, Debug)]
pub struct AppConfig {
    pub bind_addr: SocketAddr,
    pub external_api_url: Url,
    pub poll_interval_seconds: u64,
    pub public_key_pem: String,
}

impl AppConfig {
    pub fn from_env() -> Result<Self, String> {
        Self::from_lookup(|key| std::env::var(key).ok())
    }

    pub fn from_lookup<F>(lookup: F) -> Result<Self, String>
    where
        F: Fn(&str) -> Option<String>,
    {
        let bind_addr = lookup("BIND_ADDR")
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| DEFAULT_BIND_ADDR.to_string())
            .parse::<SocketAddr>()
            .map_err(|err| format!("BIND_ADDR must be a valid socket address: {err}"))?;

        let external_api_url = lookup("EXTERNAL_API_URL")
            .filter(|value| !value.trim().is_empty())
            .ok_or_else(|| "EXTERNAL_API_URL must be set and cannot be empty".to_string())
            .and_then(|value| {
                Url::parse(&value).map_err(|err| format!("EXTERNAL_API_URL is invalid: {err}"))
            })?;

        let poll_interval_seconds = lookup("POLL_INTERVAL_SECONDS")
            .filter(|value| !value.trim().is_empty())
            .map(|value| {
                value.parse::<u64>().map_err(|err| {
                    format!("POLL_INTERVAL_SECONDS must be a positive integer: {err}")
                })
            })
            .transpose()?
            .unwrap_or(DEFAULT_POLL_INTERVAL_SECONDS);
        if poll_interval_seconds == 0 {
            return Err("POLL_INTERVAL_SECONDS must be greater than 0".to_string());
        }

        let public_key_pem = lookup("RSA_PUBLIC_KEY")
            .filter(|value| !value.trim().is_empty())
            .ok_or_else(|| "RSA_PUBLIC_KEY must be set and cannot be empty".to_string())?;

        Ok(Self {
            bind_addr,
            external_api_url,
            poll_interval_seconds,
            public_key_pem,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PUBLIC_KEY: &str = "-----BEGIN PUBLIC KEY-----\nMIIB\n-----END PUBLIC KEY-----";

    #[test]
    fn uses_default_bind_addr_when_missing() {
        let config = AppConfig::from_lookup(|key| match key {
            "EXTERNAL_API_URL" => Some("https://example.com/online.json".to_string()),
            "RSA_PUBLIC_KEY" => Some(PUBLIC_KEY.to_string()),
            _ => None,
        })
        .unwrap();

        assert_eq!(config.bind_addr.to_string(), DEFAULT_BIND_ADDR);
        assert_eq!(config.poll_interval_seconds, DEFAULT_POLL_INTERVAL_SECONDS);
    }

    #[test]
    fn rejects_empty_external_api_url() {
        let err = AppConfig::from_lookup(|key| match key {
            "EXTERNAL_API_URL" => Some("".to_string()),
            "RSA_PUBLIC_KEY" => Some(PUBLIC_KEY.to_string()),
            _ => None,
        })
        .unwrap_err();

        assert!(err.contains("EXTERNAL_API_URL"));
    }

    #[test]
    fn rejects_zero_poll_interval() {
        let err = AppConfig::from_lookup(|key| match key {
            "EXTERNAL_API_URL" => Some("https://example.com/online.json".to_string()),
            "POLL_INTERVAL_SECONDS" => Some("0".to_string()),
            "RSA_PUBLIC_KEY" => Some(PUBLIC_KEY.to_string()),
            _ => None,
        })
        .unwrap_err();

        assert!(err.contains("greater than 0"));
    }
}
