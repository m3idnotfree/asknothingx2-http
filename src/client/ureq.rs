use std::{
    fmt::{Debug, Formatter, Result as FmtResult},
    pin::Pin,
    task::{Context, Poll},
};

use tower_service::Service;
use ureq::{Agent, RequestExt};

use crate::{Body, BoxedFuture, Request, Response, error::BoxError};

#[derive(Debug, Clone)]
pub struct Ureq {
    agent: Agent,
}

impl Ureq {
    pub fn new(agent: Agent) -> Self {
        Self { agent }
    }
}

impl Service<Request> for Ureq {
    type Response = Response;
    type Error = BoxError;
    type Future = UreqFuture;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request) -> Self::Future {
        let agent = self.agent.clone();

        UreqFuture(Box::pin(async move {
            let (parts, body) = req.into_parts();
            let bytes = body.collect_bytes().await?;

            let mut resp = http::Request::from_parts(parts, &bytes[..])
                .with_agent(&agent)
                .configure()
                .http_status_as_error(false)
                .run()?;

            let body = resp.body_mut().read_to_vec()?;
            Ok(resp.map(|_| Body::from(body)))
        }))
    }
}

#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct UreqFuture(BoxedFuture);

impl Future for UreqFuture {
    type Output = Result<Response, BoxError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.0.as_mut().poll(cx)
    }
}

impl Debug for UreqFuture {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_struct("UreqFuture").finish_non_exhaustive()
    }
}
