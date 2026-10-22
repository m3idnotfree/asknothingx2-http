// ```toml
// [dependencies]
// anyhow = "1"
// asknothingx2-http = { version = "0.2", features = ["ureq"] }
// http = "1"
// pollster = "1"
// serde_json = "1"
// tower = { version = "0.5", features = ["limit"] }
// tower-http = { version = "0.7", features = ["set-header", "trace"] }
// ureq = "3"
// ```

use asknothingx2_http::{Body, ClientService, client::Ureq};
use http::{
    HeaderValue,
    header::{CONTENT_TYPE, USER_AGENT},
};
use serde_json::json;
use tower::limit::ConcurrencyLimitLayer;
use tower_http::{set_header::SetRequestHeaderLayer, trace::TraceLayer};

fn main() -> anyhow::Result<()> {
    let svc = ClientService::new(Ureq::new(ureq::Agent::new_with_defaults()))
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

    let req = http::Request::post("http://example.com/users")
        .header(CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_vec(&json!({ "name": "jonh" }))?))?;

    let resp = pollster::block_on(svc.execute(req))?;
    let bytes = pollster::block_on(resp.into_body().collect_bytes())?;

    println!("{}", String::from_utf8_lossy(&bytes));

    Ok(())
}
