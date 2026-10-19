use std::{
    fmt::{Debug, Formatter, Result as FmtResult},
    pin::Pin,
    task::{Context, Poll},
};

use tower_service::Service;

use crate::{Body, Request, Response, error::BoxError};

#[derive(Debug, Clone)]
pub struct Reqwest {
    client: reqwest::Client,
}

impl Reqwest {
    pub fn new(client: reqwest::Client) -> Self {
        Self { client }
    }
}

impl Service<Request> for Reqwest {
    type Response = Response;
    type Error = BoxError;
    type Future = ReqwestFuture;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request) -> Self::Future {
        match reqwest::Request::try_from(req) {
            Ok(req) => ReqwestFuture(Inner::Sending(self.client.call(req))),
            Err(err) => ReqwestFuture(Inner::Failed(Some(err.into()))),
        }
    }
}

type Pending = <reqwest::Client as Service<reqwest::Request>>::Future;

#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct ReqwestFuture(Inner);

enum Inner {
    Sending(Pending),
    Failed(Option<BoxError>),
}

impl Future for ReqwestFuture {
    type Output = Result<Response, BoxError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match &mut self.0 {
            Inner::Sending(pending) => Pin::new(pending)
                .poll(cx)
                .map_ok(|resp| http::Response::from(resp).map(Body::from))
                .map_err(Into::into),
            Inner::Failed(err) => Poll::Ready(Err(err
                .take()
                .expect("`ReqwestFuture` polled after completion"))),
        }
    }
}

impl Debug for ReqwestFuture {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_struct("ReqwestFuture").finish_non_exhaustive()
    }
}
