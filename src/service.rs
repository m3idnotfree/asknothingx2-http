use std::{
    fmt::{Debug, Formatter, Result as FmtResult},
    pin::Pin,
    task::{Context, Poll},
};

use bytes::Bytes;
use tower::{ServiceExt, util::BoxCloneSyncService};
use tower_layer::Layer;
use tower_service::Service;

use crate::{Body, BoxedFuture, Error, Request, Response, error::BoxError};

type BoxedService = BoxCloneSyncService<Request, Response, BoxError>;

#[derive(Clone)]
pub struct ClientService(BoxedService);

impl ClientService {
    pub fn new<S, B>(service: S) -> Self
    where
        S: Service<Request, Response = http::Response<B>> + Clone + Send + Sync + 'static,
        S::Error: Into<BoxError>,
        S::Future: Send + 'static,
        B: http_body::Body<Data = Bytes> + Send + Sync + 'static,
        B::Error: Into<BoxError>,
    {
        Self(BoxedService::new(
            service
                .map_response(|resp| resp.map(Body::wrap))
                .map_err(Into::into),
        ))
    }

    pub fn layer<L, B>(self, layer: L) -> Self
    where
        L: Layer<ClientService>,
        L::Service: Service<Request, Response = http::Response<B>> + Clone + Send + Sync + 'static,
        <L::Service as Service<Request>>::Error: Into<BoxError>,
        <L::Service as Service<Request>>::Future: Send + 'static,
        B: http_body::Body<Data = Bytes> + Send + Sync + 'static,
        B::Error: Into<BoxError>,
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
