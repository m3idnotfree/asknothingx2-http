// ```toml
// [dependencies]
// anyhow = "1"
// asknothingx2-http = { version = "0.2", features = ["reqwest"] }
// http = "1"
// reqwest = { version = "0.13", features = ["json"] }
// serde_json = "1"
// tokio = { version = "1.53", features = ["full"] }
// tower = { version = "0.5", features = ["limit"] }
// tower-http = { version = "0.7", features = ["trace"] }
// ```

use asknothingx2_http::{Body, ClientService, HeaderValue, client::Reqwest, config::preset};
use tower::limit::ConcurrencyLimitLayer;
use tower_http::trace::TraceLayer;

const USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = preset::json_api(
        reqwest::Client::builder(),
        HeaderValue::from_static(USER_AGENT),
    )
    .referer(false)
    .https_only(false)
    .build()?;

    let svc = ClientService::new(Reqwest::new(client.clone()))
        .layer(ConcurrencyLimitLayer::new(10))
        .layer(TraceLayer::new_for_http());

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
