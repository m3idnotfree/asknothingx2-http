// ```toml
// [dependencies]
// anyhow = "1"
// asknothingx2-http = { version = "0.2", features = ["reqwest"] }
// http = "1"
// reqwest = "0.13"
// tokio = { version = "1.53", features = ["full"] }
// tower = { version = "0.5", features = ["limit", "util"] }
// tower-http = { version = "0.7", features = ["trace"] }
// ```
use std::sync::LazyLock;

use asknothingx2_http::{Body, ClientFuture, ClientService, HttpService, Request, client::Reqwest};
use tower::{ServiceBuilder, limit::ConcurrencyLimitLayer};
use tower_http::trace::TraceLayer;
use tower_layer::{Identity, Layer, Stack};

static HTTP_CLIENT: LazyLock<reqwest::Client> = LazyLock::new(|| {
    reqwest::Client::builder()
        .user_agent(concat!(
            env!("CARGO_PKG_NAME"),
            "/",
            env!("CARGO_PKG_VERSION"),
        ))
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .expect("failed to build the default reqwest client")
});

#[must_use]
pub struct ClientBuilder<L = Identity> {
    concurrency_limit: Option<usize>,
    layers: L,
}

impl ClientBuilder {
    pub fn new() -> Self {
        Self {
            concurrency_limit: None,
            layers: Identity::new(),
        }
    }
}

impl Default for ClientBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl<L> ClientBuilder<L> {
    pub fn concurrency_limit(mut self, max: usize) -> Self {
        self.concurrency_limit = Some(max);
        self
    }

    pub fn layer<T>(self, layer: T) -> ClientBuilder<Stack<T, L>> {
        ClientBuilder {
            concurrency_limit: self.concurrency_limit,
            layers: Stack::new(layer, self.layers),
        }
    }

    pub fn build(self) -> Client
    where
        L: Layer<Reqwest, Service: HttpService>,
    {
        self.http_client(HTTP_CLIENT.clone())
    }

    pub fn http_client(self, http_client: reqwest::Client) -> Client
    where
        L: Layer<Reqwest, Service: HttpService>,
    {
        self.service(Reqwest::new(http_client))
    }

    pub fn service<S>(self, service: S) -> Client
    where
        L: Layer<S, Service: HttpService>,
    {
        let svc = ServiceBuilder::new()
            .option_layer(self.concurrency_limit.map(ConcurrencyLimitLayer::new))
            .layer(self.layers)
            .service(service);

        Client {
            svc: ClientService::new(svc),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Client {
    svc: ClientService,
}

impl Client {
    pub fn builder() -> ClientBuilder {
        ClientBuilder::new()
    }

    pub fn send(&self, req: Request) -> ClientFuture {
        self.svc.clone().execute(req)
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = Client::builder()
        .concurrency_limit(8)
        .layer(TraceLayer::new_for_http())
        .build();

    let req = http::Request::get("http://example.com").body(Body::empty())?;
    let resp = client.send(req).await?;
    let bytes = resp.into_body().collect_bytes().await?;

    println!("{}", String::from_utf8_lossy(&bytes));

    Ok(())
}
