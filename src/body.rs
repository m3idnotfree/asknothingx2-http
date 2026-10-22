use std::{
    fmt::{Debug, Formatter, Result as FmtResult},
    pin::Pin,
    task::{Context, Poll},
};

use bytes::Bytes;
use http_body::{Frame, SizeHint};
use http_body_util::{BodyExt, combinators::BoxBody};

use crate::{Error, error::BoxError, try_downcast};

pub struct Body(Inner);

enum Inner {
    Full(Bytes),
    Boxed(BoxBody<Bytes, BoxError>),
}

impl Body {
    pub fn wrap<B>(body: B) -> Self
    where
        B: http_body::Body<Data = Bytes> + Send + Sync + 'static,
        B::Error: Into<BoxError>,
    {
        try_downcast(body)
            .unwrap_or_else(|body| Self(Inner::Boxed(body.map_err(Into::into).boxed())))
    }

    pub fn empty() -> Self {
        Self(Inner::Full(Bytes::new()))
    }

    pub fn as_bytes(&self) -> Option<&[u8]> {
        match &self.0 {
            Inner::Full(bytes) => Some(bytes),
            Inner::Boxed(_) => None,
        }
    }

    pub fn try_clone(&self) -> Option<Self> {
        match &self.0 {
            Inner::Full(bytes) => Some(Self(Inner::Full(bytes.clone()))),
            Inner::Boxed(_) => None,
        }
    }

    pub fn content_length(&self) -> Option<u64> {
        match &self.0 {
            Inner::Full(bytes) => Some(bytes.len() as u64),
            Inner::Boxed(body) => http_body::Body::size_hint(body).exact(),
        }
    }

    pub fn try_into_bytes(self) -> Result<Bytes, Self> {
        match self.0 {
            Inner::Full(bytes) => Ok(bytes),
            inner => Err(Self(inner)),
        }
    }

    pub async fn collect_bytes(self) -> Result<Bytes, Error> {
        match self.0 {
            Inner::Full(bytes) => Ok(bytes),
            Inner::Boxed(body) => Ok(body.collect().await.map_err(Error::new)?.to_bytes()),
        }
    }
}

impl Default for Body {
    fn default() -> Self {
        Self::empty()
    }
}

impl From<Bytes> for Body {
    fn from(value: Bytes) -> Self {
        Self(Inner::Full(value))
    }
}

impl From<Vec<u8>> for Body {
    fn from(value: Vec<u8>) -> Self {
        Self(Inner::Full(value.into()))
    }
}

impl From<&'static [u8]> for Body {
    fn from(value: &'static [u8]) -> Self {
        Self(Inner::Full(Bytes::from_static(value)))
    }
}

impl From<String> for Body {
    fn from(value: String) -> Self {
        Self(Inner::Full(value.into()))
    }
}

impl From<&'static str> for Body {
    fn from(value: &'static str) -> Self {
        value.as_bytes().into()
    }
}

#[cfg(feature = "reqwest")]
impl From<reqwest::Body> for Body {
    fn from(value: reqwest::Body) -> Self {
        match value.as_bytes() {
            Some(bytes) => Body::from(Bytes::copy_from_slice(bytes)),
            None => Body::wrap(value),
        }
    }
}

#[cfg(feature = "reqwest")]
impl From<Body> for reqwest::Body {
    fn from(value: Body) -> Self {
        match value.try_into_bytes() {
            Ok(bytes) => reqwest::Body::from(bytes),
            Err(body) => reqwest::Body::wrap(body),
        }
    }
}

impl Debug for Body {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match &self.0 {
            Inner::Full(bytes) => f.debug_tuple("Body").field(&bytes.len()).finish(),
            Inner::Boxed(_) => f.debug_tuple("Body").field(&"boxed").finish(),
        }
    }
}

impl http_body::Body for Body {
    type Data = Bytes;
    type Error = BoxError;

    fn poll_frame(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Self::Data>, Self::Error>>> {
        match &mut self.get_mut().0 {
            Inner::Full(bytes) if bytes.is_empty() => Poll::Ready(None),
            Inner::Full(bytes) => Poll::Ready(Some(Ok(Frame::data(std::mem::take(bytes))))),
            Inner::Boxed(body) => Pin::new(body).poll_frame(cx),
        }
    }

    fn is_end_stream(&self) -> bool {
        match &self.0 {
            Inner::Full(bytes) => bytes.is_empty(),
            Inner::Boxed(body) => body.is_end_stream(),
        }
    }

    fn size_hint(&self) -> SizeHint {
        match &self.0 {
            Inner::Full(bytes) => SizeHint::with_exact(bytes.len() as u64),
            Inner::Boxed(body) => body.size_hint(),
        }
    }
}

#[cfg(test)]
mod tests {
    use bytes::Bytes;
    use http_body::Body as _;

    use super::Body;

    #[test]
    fn as_bytes() {
        let test_body = b"test body";
        let body = Body::from(&test_body[..]);
        assert_eq!(body.as_bytes(), Some(&test_body[..]));
    }

    #[test]
    fn exact_length() {
        let empty_body = Body::empty();
        assert!(empty_body.is_end_stream());
        assert_eq!(empty_body.size_hint().exact(), Some(0));

        let bytes_body = Body::from("abc");
        assert!(!bytes_body.is_end_stream());
        assert_eq!(bytes_body.size_hint().exact(), Some(3));

        let stream_body = Body::wrap(http_body_util::Empty::<Bytes>::new());
        assert!(stream_body.is_end_stream());
        assert_eq!(stream_body.size_hint().exact(), Some(0));
    }
}
