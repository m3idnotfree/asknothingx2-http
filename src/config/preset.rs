use std::time::Duration;

use http::{HeaderMap, HeaderValue, header::ACCEPT};

use crate::config::{ApplyConfig, ClientConfig, Connection, Http, Redirect, Security};

const ACCEPT_JSON: HeaderValue = HeaderValue::from_static("application/json");

impl ClientConfig {
    /// General-purpose defaults.
    ///
    /// ## Configuration
    ///
    /// - Total timeout: 30s
    /// - User-Agent: given
    /// - Headers: none
    /// - Redirects: Follow up to 3
    /// - Connect timeout: 10s
    /// - Pool: 90s idle timeout, 20 max idle per host
    /// - Security: HTTPS-only
    pub fn general(user_agent: HeaderValue) -> Self {
        let mut config = ClientConfig::default();
        config.http.user_agent = Some(user_agent);
        config
    }

    /// For JSON APIs.
    ///
    /// Same as [`ClientConfig::general`], plus:
    /// - Headers:
    ///   - `Accept: application/json`
    pub fn json_api(user_agent: HeaderValue) -> Self {
        let mut config = Self::general(user_agent);
        config.http.default_headers.insert(ACCEPT, ACCEPT_JSON);
        config
    }

    /// For authentication flows.
    ///
    /// ## Configuration
    ///
    /// - Total timeout: 60s
    /// - User-Agent: given
    /// - Headers: none
    /// - Redirects: Do not follow
    /// - Connect timeout: 10s
    /// - Pool: 90s idle timeout, 30 max idle per host
    /// - Security: HTTPS-only
    pub fn authentication(user_agent: HeaderValue) -> Self {
        ClientConfig {
            total_timeout: Some(Duration::from_secs(60)),
            http: Http {
                user_agent: Some(user_agent),
                default_headers: HeaderMap::new(),
                redirect: Redirect::DoNotFollow,
            },
            connection: Connection {
                connect_timeout: Some(Duration::from_secs(10)),
                pool_idle_timeout: Some(Duration::from_secs(90)),
                pool_max_idle_per_host: Some(30),
            },
            security: Security::default(),
        }
    }

    /// For time-sensitive calls: give up early and keep connections ready.
    ///
    /// ## Configuration
    ///
    /// - Total timeout: 3s
    /// - User-Agent: given
    /// - Headers: none
    /// - Redirects: Do not follow
    /// - Connect timeout: 500ms
    /// - Pool: 180s idle timeout, 100 max idle per host
    /// - Security: HTTPS-only
    pub fn time_sensitive(user_agent: HeaderValue) -> Self {
        ClientConfig {
            total_timeout: Some(Duration::from_secs(3)),
            http: Http {
                user_agent: Some(user_agent),
                default_headers: HeaderMap::new(),
                redirect: Redirect::DoNotFollow,
            },
            connection: Connection {
                connect_timeout: Some(Duration::from_millis(500)),
                pool_idle_timeout: Some(Duration::from_secs(180)),
                pool_max_idle_per_host: Some(100),
            },
            security: Security::default(),
        }
    }

    /// For test environments. **Do not use in production.**
    ///
    /// ## Configuration
    ///
    /// - Total timeout: 10s
    /// - User-Agent: given
    /// - Headers: none
    /// - Redirects: Follow up to 3
    /// - Connect timeout: 3s
    /// - Pool: 5s idle timeout, 1 max idle per host
    /// - Security: **HTTP allowed** (no TLS required)
    pub fn danger_test(user_agent: HeaderValue) -> Self {
        ClientConfig {
            total_timeout: Some(Duration::from_secs(10)),
            http: Http {
                user_agent: Some(user_agent),
                default_headers: HeaderMap::new(),
                redirect: Redirect::follow(3),
            },
            connection: Connection {
                connect_timeout: Some(Duration::from_secs(3)),
                pool_idle_timeout: Some(Duration::from_secs(5)),
                pool_max_idle_per_host: Some(1),
            },
            security: insecure(),
        }
    }

    /// For debugging during development. **Do not use in production.**
    ///
    /// ## Configuration
    ///
    /// - Total timeout: 300s
    /// - User-Agent: given
    /// - Headers: none
    /// - Redirects: Follow up to 15
    /// - Connect timeout: 30s
    /// - Pool: 60s idle timeout, 1 max idle per host
    /// - Security: **HTTP allowed** (no TLS required)
    pub fn danger_debug(user_agent: HeaderValue) -> Self {
        ClientConfig {
            total_timeout: Some(Duration::from_secs(300)),
            http: Http {
                user_agent: Some(user_agent),
                default_headers: HeaderMap::new(),
                redirect: Redirect::follow(15),
            },
            connection: Connection {
                connect_timeout: Some(Duration::from_secs(30)),
                pool_idle_timeout: Some(Duration::from_secs(60)),
                pool_max_idle_per_host: Some(1),
            },
            security: insecure(),
        }
    }
}

/// Applies [`ClientConfig::general`] to `builder`.
pub fn general<T: ApplyConfig>(builder: T, user_agent: HeaderValue) -> T {
    builder.apply_config(ClientConfig::general(user_agent))
}

/// Applies [`ClientConfig::json_api`] to `builder`.
pub fn json_api<T: ApplyConfig>(builder: T, user_agent: HeaderValue) -> T {
    builder.apply_config(ClientConfig::json_api(user_agent))
}

/// Applies [`ClientConfig::authentication`] to `builder`.
pub fn authentication<T: ApplyConfig>(builder: T, user_agent: HeaderValue) -> T {
    builder.apply_config(ClientConfig::authentication(user_agent))
}

/// Applies [`ClientConfig::time_sensitive`] to `builder`.
pub fn time_sensitive<T: ApplyConfig>(builder: T, user_agent: HeaderValue) -> T {
    builder.apply_config(ClientConfig::time_sensitive(user_agent))
}

/// Applies [`ClientConfig::danger_test`] to `builder`.
pub fn danger_test<T: ApplyConfig>(builder: T, user_agent: HeaderValue) -> T {
    builder.apply_config(ClientConfig::danger_test(user_agent))
}

/// Applies [`ClientConfig::danger_debug`] to `builder`.
pub fn danger_debug<T: ApplyConfig>(builder: T, user_agent: HeaderValue) -> T {
    builder.apply_config(ClientConfig::danger_debug(user_agent))
}

fn insecure() -> Security {
    Security { https_only: false }
}

#[cfg(test)]
mod tests {
    use super::*;

    const UA: HeaderValue = HeaderValue::from_static(concat!(
        env!("CARGO_PKG_NAME"),
        "/",
        env!("CARGO_PKG_VERSION"),
    ));

    #[test]
    fn require_https() {
        for config in [
            ClientConfig::general(UA),
            ClientConfig::json_api(UA),
            ClientConfig::authentication(UA),
            ClientConfig::time_sensitive(UA),
        ] {
            assert!(config.security.https_only);
        }
    }

    #[test]
    fn allow_http() {
        assert!(!ClientConfig::danger_test(UA).security.https_only);
        assert!(!ClientConfig::danger_debug(UA).security.https_only);
    }
}
