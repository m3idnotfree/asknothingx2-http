//! Client-independent HTTP client settings.
//!
//! # Applying to a client
//!
//! ## reqwest (feature `reqwest`)
//!
//! ```
//! # #[cfg(feature = "reqwest")] {
//! use asknothingx2_http::{HeaderValue, config::preset};
//!
//! let user_agent = HeaderValue::from_static(concat!(
//!     env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),
//! ));
//!
//! let client = preset::json_api(reqwest::Client::builder(), user_agent).build()?;
//! # }
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ## ureq (feature `ureq`)
//!
//! ```
//! # #[cfg(feature = "ureq")] {
//! use asknothingx2_http::{HeaderValue, config::preset};
//!
//! let user_agent = HeaderValue::from_static(concat!(
//!     env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),
//! ));
//!
//! let agent = preset::json_api(ureq::Agent::config_builder(), user_agent)
//!     .build()
//!     .new_agent();
//! # }
//! ```
//!
//! # Customizing
//!
//! ```
//! # #[cfg(feature = "reqwest")] {
//! use std::time::Duration;
//!
//! use asknothingx2_http::{
//!     HeaderValue,
//!     config::{ApplyConfig, ClientConfig, Redirect, preset},
//! };
//!
//! let mut config = ClientConfig::json_api(HeaderValue::from_static(concat!(
//!     env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),
//! )));
//!
//! config.total_timeout = Some(Duration::from_secs(10));
//! config.http.redirect = Redirect::DoNotFollow;
//!
//! let client = reqwest::Client::builder().apply_config(config)
//!     .read_timeout(Duration::from_secs(5))
//!     .referer(false)
//!     .build()?;
//! # }
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
pub mod preset;
#[cfg(feature = "reqwest")]
pub mod reqwest;
#[cfg(feature = "ureq")]
pub mod ureq;

use std::{num::NonZeroU32, time::Duration};

use http::{HeaderMap, HeaderValue};

pub trait ApplyConfig: Sized {
    fn apply_config(self, config: ClientConfig) -> Self;
}

/// Client-independent HTTP client settings.
///
/// ## Defaults
///
/// - Total timeout: 30s
/// - User-Agent: none
/// - Headers: none
/// - Redirects: Follow up to 3
/// - Connect timeout: 10s
/// - Pool: 90s idle timeout, 20 max idle per host
/// - Security: HTTPS-only
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct ClientConfig {
    pub total_timeout: Option<Duration>,
    pub http: Http,
    pub connection: Connection,
    pub security: Security,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            total_timeout: Some(Duration::from_secs(30)),
            http: Http::default(),
            connection: Connection::default(),
            security: Security::default(),
        }
    }
}

impl ClientConfig {
    pub fn new() -> Self {
        ClientConfig::default()
    }
}

/// Header and redirect settings.
///
/// ## Defaults
///
/// - User-Agent: none
/// - Headers: none
/// - Redirects: Follow up to 3
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct Http {
    pub user_agent: Option<HeaderValue>,
    pub default_headers: HeaderMap,
    pub redirect: Redirect,
}

impl Default for Http {
    fn default() -> Self {
        Self {
            user_agent: None,
            default_headers: HeaderMap::new(),
            redirect: Redirect::follow(3),
        }
    }
}

/// Connection and pool settings.
///
/// ## Defaults
///
/// - Connect timeout: 10s
/// - Pool: 90s idle timeout, 20 max idle per host
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct Connection {
    pub connect_timeout: Option<Duration>,
    pub pool_idle_timeout: Option<Duration>,
    pub pool_max_idle_per_host: Option<usize>,
}

impl Default for Connection {
    fn default() -> Self {
        Self {
            connect_timeout: Some(Duration::from_secs(10)),
            pool_idle_timeout: Some(Duration::from_secs(90)),
            pool_max_idle_per_host: Some(20),
        }
    }
}

/// Security settings.
///
/// ## Defaults
///
/// - Security: HTTPS-only
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct Security {
    pub https_only: bool,
}

impl Default for Security {
    fn default() -> Self {
        Self { https_only: true }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Redirect {
    DoNotFollow,
    Follow { max: NonZeroU32 },
}

impl Redirect {
    /// Follow at most `max` redirects. `0` means [`Redirect::DoNotFollow`].
    pub const fn follow(max: u32) -> Self {
        match NonZeroU32::new(max) {
            Some(max) => Redirect::Follow { max },
            None => Redirect::DoNotFollow,
        }
    }
}
