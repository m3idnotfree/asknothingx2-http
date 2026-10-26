use bytes::BytesMut;
use http::{
    HeaderMap, HeaderName, HeaderValue,
    header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
};

const CLIENT_ID: HeaderName = HeaderName::from_static("client-id");
const CLIENT_SECRET: HeaderName = HeaderName::from_static("client-secret");

const APPLICATION_JSON: HeaderValue = HeaderValue::from_static("application/json");
const FORM_URLENCODED: HeaderValue = HeaderValue::from_static("application/x-www-form-urlencoded");

const BEARER: &[u8] = b"Bearer ";

mod private {
    use http::HeaderMap;

    pub trait Sealed {}

    impl Sealed for HeaderMap {}
}

pub trait HeaderMapExt: private::Sealed {
    /// Sets `client-id`.
    fn client_id(&mut self, client_id: HeaderValue) -> &mut Self;
    /// Sets `client-secret` (sensitive).
    fn client_secret(&mut self, client_secret: HeaderValue) -> &mut Self;
    /// Sets `accept: application/json`.
    fn accept_json(&mut self) -> &mut Self;
    /// Sets `content-type: application/json`.
    fn content_type_json(&mut self) -> &mut Self;
    /// Sets `content-type: application/x-www-form-urlencoded`.
    fn content_type_form_urlencoded(&mut self) -> &mut Self;
    /// Sets `authorization` (sensitive).
    fn authorization(&mut self, value: HeaderValue) -> &mut Self;
    /// Sets `authorization: Bearer <token>` (sensitive).
    fn bearer_token(&mut self, token: HeaderValue) -> &mut Self;
}

impl HeaderMapExt for HeaderMap {
    fn client_id(&mut self, client_id: HeaderValue) -> &mut Self {
        self.insert(CLIENT_ID, client_id);
        self
    }

    fn client_secret(&mut self, mut client_secret: HeaderValue) -> &mut Self {
        client_secret.set_sensitive(true);
        self.insert(CLIENT_SECRET, client_secret);
        self
    }

    fn accept_json(&mut self) -> &mut Self {
        self.insert(ACCEPT, APPLICATION_JSON);
        self
    }

    fn content_type_json(&mut self) -> &mut Self {
        self.insert(CONTENT_TYPE, APPLICATION_JSON);
        self
    }

    fn content_type_form_urlencoded(&mut self) -> &mut Self {
        self.insert(CONTENT_TYPE, FORM_URLENCODED);
        self
    }

    fn authorization(&mut self, mut value: HeaderValue) -> &mut Self {
        value.set_sensitive(true);
        self.insert(AUTHORIZATION, value);
        self
    }

    fn bearer_token(&mut self, token: HeaderValue) -> &mut Self {
        let mut buf = BytesMut::with_capacity(BEARER.len() + token.len());
        buf.extend_from_slice(BEARER);
        buf.extend_from_slice(token.as_bytes());

        let value = HeaderValue::from_maybe_shared(buf.freeze())
            .expect("bearer token should be a valid header value");

        self.authorization(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bearer_token() {
        let mut headers = HeaderMap::new();
        headers.bearer_token(HeaderValue::from_static("token"));

        let value = &headers[AUTHORIZATION];
        assert_eq!(value, "Bearer token");
        assert!(value.is_sensitive());
    }

    #[test]
    fn replace_values() {
        let mut headers = HeaderMap::new();
        headers.append(ACCEPT, HeaderValue::from_static("text/html"));
        headers.append(ACCEPT, HeaderValue::from_static("text/plain"));
        headers.accept_json();

        assert_eq!(headers.get_all(ACCEPT).iter().count(), 1);
        assert_eq!(headers[ACCEPT], "application/json");
    }
}
