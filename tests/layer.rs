use std::sync::{Arc, Mutex};

use asknothingx2_http::{Body, ClientService, Request, Response};
use bytes::Bytes;
use http::{Request as HttpReq, Response as HttpResp};
use http_body_util::Full;
use tokio_test::assert_ready_ok;
use tower::{ServiceBuilder, ServiceExt};
use tower_layer::{Layer, layer_fn};
use tower_test::mock;

#[tokio::test]
async fn changed_response_body() {
    let (inner, mut handle) = mock::pair::<Request, Response>();
    let change_body = layer_fn(|inner: ClientService| {
        inner.map_response(|resp| resp.map(|_| Full::<Bytes>::from("changed")))
    });

    let mut svc = mock::Spawn::new(ClientService::new(inner).layer(change_body));

    assert_ready_ok!(svc.poll_ready());
    let fut = svc.call(HttpReq::get("/abc").body(Body::empty()).unwrap());

    let (_, send) = handle.next_request().await.unwrap();
    send.send_response(HttpResp::new(Body::from("original")));

    let resp = fut.await.unwrap();
    let body: Body = resp.into_body();

    assert!(body.as_bytes().is_none());
    assert_eq!(body.collect_bytes().await.unwrap(), "changed");
}

#[tokio::test]
async fn last_layer_sees_request_first() {
    let (inner, mut handle) = mock::pair::<Request, Response>();
    let seen = Seen::default();

    let mut svc = mock::Spawn::new(
        ClientService::new(inner)
            .layer(mark(&seen, "first"))
            .layer(mark(&seen, "second")),
    );

    assert_ready_ok!(svc.poll_ready());

    let fut = svc.call(HttpReq::get("/abc").body(Body::empty()).unwrap());

    let (_, send) = handle.next_request().await.unwrap();
    send.send_response(HttpResp::new(Body::empty()));

    fut.await.unwrap();
    assert_eq!(*seen.lock().unwrap(), ["second", "first"])
}

#[tokio::test]
async fn keep_service_builder_layer_order() {
    let (inner, mut handle) = mock::pair::<Request, Response>();
    let seen = Seen::default();

    let mut svc = mock::Spawn::new(
        ClientService::new(inner).layer(
            ServiceBuilder::new()
                .layer(mark(&seen, "first"))
                .layer(mark(&seen, "second")),
        ),
    );

    assert_ready_ok!(svc.poll_ready());

    let fut = svc.call(HttpReq::get("/abc").body(Body::empty()).unwrap());

    let (_, send) = handle.next_request().await.unwrap();
    send.send_response(HttpResp::new(Body::empty()));

    fut.await.unwrap();
    assert_eq!(*seen.lock().unwrap(), ["first", "second"])
}

type Seen = Arc<Mutex<Vec<&'static str>>>;

fn mark(seen: &Seen, name: &'static str) -> impl Layer<ClientService, Service = ClientService> {
    let seen = seen.clone();
    layer_fn(move |inner: ClientService| {
        let seen = seen.clone();
        ClientService::new(inner.map_request(move |req: Request| {
            seen.lock().unwrap().push(name);
            req
        }))
    })
}
