// ```toml
// [dependencies]
// anyhow = "1"
// asknothingx2-http = "0.2"
// bytes = "1"
// http = "1"
// tower-service = "0.3"
// ```

use std::{
    pin::Pin,
    task::{Context, Poll},
};

use asknothingx2_http::{Body, ClientService, Request, Response};
use bytes::Bytes;
use tower_service::Service;

type BoxError = Box<dyn std::error::Error + Send + Sync>;

#[derive(Clone)]
struct CustomClient {
    base_uri: String,
}

impl Service<Request> for CustomClient {
    type Response = Response;
    type Error = BoxError;
    type Future = Pin<Box<dyn Future<Output = Result<Response, BoxError>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request) -> Self::Future {
        let base_uri = self.base_uri.clone();

        Box::pin(async move {
            let (mut parts, body) = req.into_parts();
            let bytes = body.collect_bytes().await?;

            let path = parts.uri.path_and_query().map_or("/", |path| path.as_str());
            parts.uri = format!("{base_uri}{path}").parse()?;

            let resp = send(http::Request::from_parts(parts, bytes))?;

            Ok(resp.map(Body::from))
        })
    }
}

fn send(req: http::Request<Bytes>) -> Result<http::Response<Bytes>, std::io::Error> {
    let text = format!("{} {}", req.method(), req.uri());
    Ok(http::Response::new(Bytes::from(text)))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = ClientService::new(CustomClient {
        base_uri: "http://example.com".into(),
    });

    let req = http::Request::get("/users").body(Body::empty())?;

    let resp = client.execute(req).await?;
    let bytes = resp.into_body().collect_bytes().await?;

    println!("{}", String::from_utf8_lossy(&bytes));

    Ok(())
}
