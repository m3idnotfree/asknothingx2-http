use asknothingx2_http::{Body, ClientService, client::Ureq};
use http::Request;
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{method, path},
};

#[tokio::test(flavor = "current_thread")]
async fn request_and_response() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/abc"))
        .respond_with(ResponseTemplate::new(200).set_body_string("ok"))
        .expect(1)
        .mount(&server)
        .await;

    let req = Request::post(format!("{}/abc", server.uri()))
        .header("name", "jonh")
        .body(Body::from("deo"))
        .unwrap();

    let resp = service().execute(req).await.unwrap();

    assert_eq!(resp.status(), 200);
    assert_eq!(resp.into_body().collect_bytes().await.unwrap(), "ok");

    let received = &server.received_requests().await.unwrap()[0];
    assert_eq!(received.headers["name"], "jonh");
    assert_eq!(received.body, b"deo");
}

fn service() -> ClientService {
    ClientService::new(Ureq::new(ureq::Agent::new_with_defaults()))
}
