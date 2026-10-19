// ```toml
// [dependencies]
// anyhow = "1"
// asknothingx2-http = { version = "0.1", features = ["reqwest"] }
// http = "1"
// reqwest = { version = "0.13", features = ["json"] }
// serde_json = "1"
// tokio = { version = "1.53", features = ["full"] }
// tower = { version = "0.5", features = ["limit"] }
// tower-http = { version = "0.7", features = ["set-header", "trace"] }
// ```

use asknothingx2_http::{Body, ClientService, client::Reqwest};
use http::{HeaderValue, header::USER_AGENT};
use tower::limit::ConcurrencyLimitLayer;
use tower_http::{set_header::SetRequestHeaderLayer, trace::TraceLayer};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = reqwest::Client::new();

    let svc = ClientService::new(Reqwest::new(client.clone()))
        .layer(ConcurrencyLimitLayer::new(10))
        .layer(TraceLayer::new_for_http())
        .layer(SetRequestHeaderLayer::overriding(
            USER_AGENT,
            HeaderValue::from_static(concat!(
                env!("CARGO_PKG_NAME"),
                "/",
                env!("CARGO_PKG_VERSION"),
            )),
        ));

    let req = client
        .post("http://example.com/users")
        .json(&serde_json::json!({ "name": "jonh" }))
        .build()?;
    let req = http::Request::<reqwest::Body>::try_from(req)?.map(Body::from);

    let resp = svc.execute(req).await?;
    let bytes = resp.into_body().collect_bytes().await?;

    println!("{}", String::from_utf8_lossy(&bytes));

    Ok(())
}
