//! | Setting | ureq | [`ClientConfig::default`] |
//! |---|---|---|
//! | Total timeout | none | 30s |
//! | Connect timeout | none | 10s |
//! | HTTPS-only | no | yes |
//! | Redirects | follow up to 10 | follow up to 3 |
//! | Pool idle timeout | 15s | 90s |
//! | Max idle per host | 3 | 20 |
//! | Max idle for all hosts | 10 | max(10, max idle per host) |
//! | User-Agent | `ureq/<version>` | none |
//! | Default headers | `Accept: */*` | none |
//!
//! ```
//! use asknothingx2_http::{HeaderValue, config::preset};
//!
//! let user_agent = HeaderValue::from_static(concat!(
//!     env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),
//! ));
//!
//! let agent = preset::json_api(ureq::Agent::config_builder(), user_agent)
//!     .build()
//!     .new_agent();
//! ```
use http::{HeaderMap, Request, Response, header::USER_AGENT};
use ureq::{
    Body, SendBody,
    config::ConfigBuilder,
    middleware::{Middleware, MiddlewareNext},
    typestate::AgentScope,
};

use super::{ApplyConfig, ClientConfig, Connection, Http, Redirect, Security};

const UREQ_MAX_IDLE_CONNECTIONS: usize = 10;

impl ApplyConfig for ConfigBuilder<AgentScope> {
    fn apply_config(self, config: ClientConfig) -> Self {
        apply(config, self)
    }
}

fn apply(
    config: ClientConfig,
    mut builder: ConfigBuilder<AgentScope>,
) -> ConfigBuilder<AgentScope> {
    let ClientConfig {
        total_timeout,
        http,
        connection,
        security,
    } = config;

    if let Some(timeout) = total_timeout {
        builder = builder.timeout_global(Some(timeout));
    }

    builder = apply_http(http, builder);
    builder = apply_connection(connection, builder);
    apply_security(security, builder)
}

fn apply_http(http: Http, mut builder: ConfigBuilder<AgentScope>) -> ConfigBuilder<AgentScope> {
    let Http {
        user_agent,
        default_headers: mut headers,
        redirect,
    } = http;

    if let Some(user_agent) = user_agent {
        match user_agent.to_str() {
            Ok(user_agent) => builder = builder.user_agent(user_agent),
            Err(_) => {
                headers.insert(USER_AGENT, user_agent);
            }
        }
    }

    if !headers.is_empty() {
        builder = builder.middleware(DefaultHeaders(headers));
    }

    let max_redirects = match redirect {
        Redirect::DoNotFollow => 0,
        Redirect::Follow { max } => max.get(),
    };

    builder.max_redirects(max_redirects)
}

fn apply_connection(
    connection: Connection,
    mut builder: ConfigBuilder<AgentScope>,
) -> ConfigBuilder<AgentScope> {
    let Connection {
        connect_timeout,
        pool_idle_timeout,
        pool_max_idle_per_host,
    } = connection;

    if let Some(timeout) = connect_timeout {
        builder = builder.timeout_connect(Some(timeout));
    }
    if let Some(timeout) = pool_idle_timeout {
        builder = builder.max_idle_age(timeout);
    }
    if let Some(max) = pool_max_idle_per_host {
        let total = UREQ_MAX_IDLE_CONNECTIONS.max(max);
        builder = builder
            .max_idle_connections_per_host(max)
            .max_idle_connections(total);
    }

    builder
}

fn apply_security(
    security: Security,
    builder: ConfigBuilder<AgentScope>,
) -> ConfigBuilder<AgentScope> {
    let Security { https_only } = security;

    builder.https_only(https_only)
}

struct DefaultHeaders(HeaderMap);

impl Middleware for DefaultHeaders {
    fn handle(
        &self,
        mut request: Request<SendBody>,
        next: MiddlewareNext,
    ) -> Result<Response<Body>, ureq::Error> {
        let headers = request.headers_mut();
        for name in self.0.keys() {
            if !headers.contains_key(name) {
                for value in self.0.get_all(name) {
                    headers.append(name.clone(), value.clone());
                }
            }
        }

        next.handle(request)
    }
}

#[cfg(test)]
mod tests {
    use http::HeaderValue;
    use ureq::config::Config;

    use crate::config::preset;

    use super::*;

    const UA: HeaderValue = HeaderValue::from_static(concat!(
        env!("CARGO_PKG_NAME"),
        "/",
        env!("CARGO_PKG_VERSION"),
    ));

    #[test]
    fn ureq_max_idle_connections_default() {
        assert_eq!(
            UREQ_MAX_IDLE_CONNECTIONS,
            Config::default().max_idle_connections()
        );
    }

    #[test]
    fn default_config_build() {
        apply(ClientConfig::default(), Config::builder())
            .build()
            .new_agent();
    }

    #[test]
    fn presets_build() {
        preset::general(Config::builder(), UA).build().new_agent();
        preset::json_api(Config::builder(), UA).build().new_agent();
        preset::authentication(Config::builder(), UA)
            .build()
            .new_agent();
        preset::time_sensitive(Config::builder(), UA)
            .build()
            .new_agent();
        preset::danger_test(Config::builder(), UA)
            .build()
            .new_agent();
        preset::danger_debug(Config::builder(), UA)
            .build()
            .new_agent();
    }
}
