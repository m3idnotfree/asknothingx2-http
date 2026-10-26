//! | Setting | reqwest | [`ClientConfig::default`] |
//! |---|---|---|
//! | Total timeout | none | 30s |
//! | Connect timeout | none | 10s |
//! | HTTPS-only | no | yes |
//! | Redirects | follow up to 10 | follow up to 3 |
//! | Pool idle timeout | 90s | 90s |
//! | Max idle per host | `usize::MAX` | 20 |
//! | User-Agent | none | none |
//! | Default headers | `Accept: */*` | none |
//!
//! ```
//! use asknothingx2_http::{HeaderValue, config::preset};
//!
//! let user_agent = HeaderValue::from_static(concat!(
//!     env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),
//! ));
//!
//! let client = preset::json_api(reqwest::Client::builder(), user_agent).build()?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
use reqwest::{ClientBuilder, redirect::Policy};

use super::{ApplyConfig, ClientConfig, Connection, Http, Redirect, Security};

impl ApplyConfig for ClientBuilder {
    fn apply_config(self, config: ClientConfig) -> Self {
        apply(config, self)
    }
}

fn apply(config: ClientConfig, mut builder: ClientBuilder) -> ClientBuilder {
    let ClientConfig {
        total_timeout,
        http,
        connection,
        security,
    } = config;

    if let Some(timeout) = total_timeout {
        builder = builder.timeout(timeout);
    }

    builder = apply_http(http, builder);
    builder = apply_connection(connection, builder);
    apply_security(security, builder)
}

fn apply_http(http: Http, builder: ClientBuilder) -> ClientBuilder {
    let Http {
        user_agent,
        default_headers,
        redirect,
    } = http;

    let mut builder = builder.default_headers(default_headers);
    if let Some(user_agent) = user_agent {
        builder = builder.user_agent(user_agent);
    }

    let policy = match redirect {
        Redirect::DoNotFollow => Policy::none(),
        Redirect::Follow { max } => {
            Policy::limited(usize::try_from(max.get()).unwrap_or(usize::MAX))
        }
    };

    builder.redirect(policy)
}

fn apply_connection(connection: Connection, mut builder: ClientBuilder) -> ClientBuilder {
    let Connection {
        connect_timeout,
        pool_idle_timeout,
        pool_max_idle_per_host,
    } = connection;

    if let Some(timeout) = connect_timeout {
        builder = builder.connect_timeout(timeout);
    }
    if let Some(timeout) = pool_idle_timeout {
        builder = builder.pool_idle_timeout(timeout);
    }
    if let Some(max) = pool_max_idle_per_host {
        builder = builder.pool_max_idle_per_host(max);
    }

    builder
}

fn apply_security(security: Security, builder: ClientBuilder) -> ClientBuilder {
    let Security { https_only } = security;

    builder.https_only(https_only)
}

#[cfg(test)]
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use std::time::Duration;

    use http::HeaderValue;
    use reqwest::Client;

    use crate::config::preset;

    use super::*;

    const UA: HeaderValue = HeaderValue::from_static(concat!(
        env!("CARGO_PKG_NAME"),
        "/",
        env!("CARGO_PKG_VERSION"),
    ));

    #[test]
    fn default_config_build() {
        apply(ClientConfig::default(), Client::builder())
            .build()
            .unwrap();
    }

    #[test]
    fn custom_config_build() {
        let mut config = ClientConfig::default();
        config.total_timeout = None;
        config.http.redirect = Redirect::DoNotFollow;
        config.connection.connect_timeout = Some(Duration::from_secs(3));
        config.security.https_only = false;

        apply(config, Client::builder()).build().unwrap();
    }

    #[test]
    fn build() {
        preset::general(Client::builder(), UA).build().unwrap();
        preset::json_api(Client::builder(), UA).build().unwrap();
        preset::authentication(Client::builder(), UA)
            .build()
            .unwrap();
        preset::time_sensitive(Client::builder(), UA)
            .build()
            .unwrap();
        preset::danger_test(Client::builder(), UA).build().unwrap();
        preset::danger_debug(Client::builder(), UA).build().unwrap();
    }
}
