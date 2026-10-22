//! Swap HTTP clients without changing your code.
//!
//! # Clients
//!
//! - [`client::Reqwest`]
//! - [`client::Ureq`]
//!
#![doc = concat!(
    "See [examples](https://docs.rs/crate/asknothingx2-http/",
    env!("CARGO_PKG_VERSION"),
    "/source/examples)"
)]
//!
//! ## Reqwest
//!
//! reqwest already uses `tower-http` for decompression.
//! Enable these reqwest features instead of adding `DecompressionLayer`.
//!
//! - `gzip`
//! - `brotli`
//! - `zstd`
//! - `deflate`
//!
//! Use `reqwest::redirect::Policy` instead of adding `FollowRedirectLayer`.
//!
//! For `form` and `multipart`, build the request with `reqwest`
//! and convert it to [`Request`].
//!
//! ```
//! # #[cfg(feature = "reqwest")] {
//! use asknothingx2_http::Body;
//!
//! let req = reqwest::Client::new()
//!     .post("https://example.com/login")
//!     .form(&[("name", "john")])
//!     .build()?;
//!
//! let req = http::Request::<reqwest::Body>::try_from(req)?.map(Body::from);
//! # }
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! # Layers
//!
//! Each layer wraps the ones before it, so the layer you add last runs first.
//!
//! ```text
//! ClientService::new(client).layer(a).layer(b)
//!
//! request:  b -> a -> client
//! response: b <- a <- client
//! ```
//!
//! # Error handling
//!
//! ```
//! use asknothingx2_http::Error;
//! use tower::timeout::error::Elapsed;
//!
//! fn is_timeout(err: &Error) -> bool {
//!     // From reqwest's timeout
//!     if let Some(err) = err.downcast_ref::<reqwest::Error>() {
//!         return err.is_timeout();
//!     }
//!     // From tower's `TimeoutLayer`
//!     err.is::<Elapsed>()
//! }
//! ```
#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(any(feature = "reqwest", feature = "ureq"))]
pub mod client;

mod body;
mod error;
mod service;

pub use body::Body;
pub use error::Error;
pub use service::{ClientFuture, ClientService, HttpService};

pub(crate) type BoxedFuture =
    std::pin::Pin<Box<dyn Future<Output = Result<Response, crate::error::BoxError>> + Send>>;

pub type Request = http::Request<Body>;
pub type Response = http::Response<Body>;

fn try_downcast<T, U>(value: U) -> Result<T, U>
where
    T: 'static,
    U: Send + 'static,
{
    let mut value = Some(value);
    if let Some(value) = <dyn std::any::Any>::downcast_mut::<Option<T>>(&mut value) {
        Ok(value.take().unwrap())
    } else {
        Err(value.unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::try_downcast;

    #[test]
    fn test_try_downcast() {
        assert_eq!(try_downcast::<i32, _>(5_u32), Err(5_u32));
        assert_eq!(try_downcast::<i32, _>(5_i32), Ok(5_i32));
    }
}
