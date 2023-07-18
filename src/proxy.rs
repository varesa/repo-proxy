use std::net::SocketAddr;
use hudsucker::async_trait::async_trait;
use hudsucker::certificate_authority::RcgenAuthority;
use hudsucker::{HttpHandler, NoopHandler};
use hudsucker::hyper::client::HttpConnector;
use hyper_rustls::HttpsConnector;
use crate::ca::Ca;

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to install CTRL+C signal handler");
}

#[derive(Clone)]
struct Handler;

#[async_trait]
impl HttpHandler for Handler {

}

pub struct Proxy  {
    hudsucker: hudsucker::Proxy<HttpsConnector<HttpConnector>, RcgenAuthority, Handler, NoopHandler>
}

impl Proxy {
    pub fn try_new(ca: Ca) -> Result<Self, anyhow::Error> {
        let rcgen = RcgenAuthority::new(ca.private_key, ca.certificate, 1_000)?;
        let proxy = hudsucker::Proxy::builder()
            .with_addr(SocketAddr::from(([0,0,0,0], 8777)))
            .with_rustls_client()
            .with_ca(rcgen)
            .with_http_handler(Handler)
            .build();

        Ok(Self {
            hudsucker: proxy,
        })
    }

    pub async fn start(self) -> Result<(), anyhow::Error> {
        self.hudsucker.start(shutdown_signal()).await?;
        Ok(())
    }
}