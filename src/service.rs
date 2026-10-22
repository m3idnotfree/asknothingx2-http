use std::{
    fmt::{Debug, Formatter, Result as FmtResult},
    pin::Pin,
    task::{Context, Poll},
};

use bytes::Bytes;
use tower::{ServiceExt, util::BoxCloneSyncService};
use tower_layer::Layer;
use tower_service::Service;

use crate::{Body, BoxedFuture, Error, Request, Response, error::BoxError, try_downcast};

type BoxedService = BoxCloneSyncService<Request, Response, BoxError>;

pub trait HttpService:
    Service<
        Request,
        Response = http::Response<Self::ResBody>,
        Error: Into<BoxError>,
        Future: Send + 'static,
    > + Clone
    + Send
    + Sync
    + 'static
{
    type ResBody: http_body::Body<Data = Bytes, Error: Into<BoxError>> + Send + Sync + 'static;
}

impl<S, B> HttpService for S
where
    S: Service<
            Request,
            Response = http::Response<B>,
            Error: Into<BoxError>,
            Future: Send + 'static,
        > + Clone
        + Send
        + Sync
        + 'static,
    B: http_body::Body<Data = Bytes, Error: Into<BoxError>> + Send + Sync + 'static,
{
    type ResBody = B;
}

#[derive(Clone)]
pub struct ClientService(BoxedService);

impl ClientService {
    pub fn new<S: HttpService>(service: S) -> Self {
        try_downcast(service).unwrap_or_else(|service: S| {
            Self(BoxedService::new(
                service
                    .map_response(|resp| resp.map(Body::wrap))
                    .map_err(Into::into),
            ))
        })
    }

    pub fn layer<L>(self, layer: L) -> Self
    where
        L: Layer<ClientService, Service: HttpService>,
    {
        Self::new(layer.layer(self))
    }

    pub fn execute(self, req: Request) -> ClientFuture {
        ClientFuture(Box::pin(self.0.oneshot(req)))
    }
}

impl Service<Request> for ClientService {
    type Response = Response;
    type Error = Error;
    type Future = ClientFuture;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.0.poll_ready(cx).map_err(Error::new)
    }

    fn call(&mut self, req: Request) -> Self::Future {
        ClientFuture(self.0.call(req))
    }
}

impl Debug for ClientService {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_struct("ClientService").finish_non_exhaustive()
    }
}

#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct ClientFuture(BoxedFuture);

impl Future for ClientFuture {
    type Output = Result<Response, Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.0.as_mut().poll(cx).map_err(Error::new)
    }
}

impl Debug for ClientFuture {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_struct("ClientFuture").finish_non_exhaustive()
    }
}
