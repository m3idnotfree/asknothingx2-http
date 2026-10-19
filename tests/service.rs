use asknothingx2_http::{Body, ClientService, Request, Response};
use http::{Request as HttpReq, Response as HttpRes};
use tokio_test::{assert_pending, assert_ready, assert_ready_err, assert_ready_ok};
use tower_test::mock;

#[tokio::test(flavor = "current_thread")]
async fn request_and_response() {
    let (mut svc, mut handle) = new_service();

    assert_ready!(svc.poll_ready()).unwrap();

    let resp = svc.call(HttpReq::get("/abc").body(Body::empty()).unwrap());

    let (req, send) = handle.next_request().await.unwrap();
    assert_eq!(req.uri(), "/abc");
    send.send_response(HttpRes::new(Body::from("world")));

    let resp = resp.await.unwrap();
    assert_eq!(resp.body().as_bytes().unwrap(), b"world");
}

#[tokio::test(flavor = "current_thread")]
async fn poll_ready_waits_for_inner() {
    let (mut svc, mut handle) = new_service();

    handle.allow(0);
    assert_pending!(svc.poll_ready());

    handle.allow(1);
    assert!(svc.is_woken());
    assert_ready_ok!(svc.poll_ready());
}

#[tokio::test(flavor = "current_thread")]
async fn poll_ready_returns_inner_error() {
    let (mut svc, mut handle) = new_service();
    handle.allow(0);
    handle.send_error("foobar");

    assert_eq!(assert_ready_err!(svc.poll_ready()).to_string(), "foobar");
}

pub fn new_service() -> (mock::Spawn<ClientService>, mock::Handle<Request, Response>) {
    let (mock, handle) = mock::pair::<Request, Response>();
    (mock::Spawn::new(ClientService::new(mock)), handle)
}
