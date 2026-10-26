// ```toml
// [dependencies]
// anyhow = "1"
// asknothingx2-http = { version = "0.2", features = ["ureq"] }
// http = "1"
// pollster = "1"
// serde_json = "1"
// tower = { version = "0.5", features = ["limit"] }
// tower-http = { version = "0.7", features = ["trace"] }
// ureq = "3"
// ```

use asknothingx2_http::{
    Body, ClientService, HeaderMapExt, HeaderValue, client::Ureq, config::preset,
};
use serde_json::json;
use tower::limit::ConcurrencyLimitLayer;
use tower_http::trace::TraceLayer;
use ureq::Agent;

const USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

fn main() -> anyhow::Result<()> {
    let agent = preset::json_api(
        Agent::config_builder(),
        HeaderValue::from_static(USER_AGENT),
    )
    .max_idle_connections(50)
    .https_only(false)
    .build()
    .new_agent();

    let svc = ClientService::new(Ureq::new(agent))
        .layer(ConcurrencyLimitLayer::new(10))
        .layer(TraceLayer::new_for_http());

    let mut req = http::Request::post("http://example.com/users")
        .body(Body::from(serde_json::to_vec(&json!({ "name": "jonh" }))?))?;

    req.headers_mut().content_type_json();

    let resp = pollster::block_on(svc.execute(req))?;
    let bytes = pollster::block_on(resp.into_body().collect_bytes())?;

    println!("{}", String::from_utf8_lossy(&bytes));

    Ok(())
}
