#[cfg(feature = "reqwest")]
mod reqwest;
#[cfg(feature = "ureq")]
mod ureq;

#[cfg(feature = "reqwest")]
pub use reqwest::{Reqwest, ReqwestFuture};
#[cfg(feature = "ureq")]
pub use ureq::{Ureq, UreqFuture};
