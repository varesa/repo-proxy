use std::net::SocketAddr;
use hudsucker::async_trait::async_trait;
use hudsucker::certificate_authority::RcgenAuthority;
use hudsucker::{HttpContext, HttpHandler, NoopHandler, RequestOrResponse};
use hudsucker::hyper::client::HttpConnector;
use hudsucker::hyper::{Body, Method, Request, Response};
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
    async fn handle_request(
        &mut self,
        _ctx: &HttpContext,
        req: Request<Body>,
    ) -> RequestOrResponse {
        if req.method() == Method::CONNECT {
            return req.into(); // We are not interested in the TCP proxy layer -> ignore
        }
        //println!("{:?}", req);

        let uri = req.uri();
        if uri.path().ends_with(".rpm") {
            println!("PACKAGE: {uri}");
        } else if uri.path().contains("repodata") {
            println!("REPODATA: {uri}");
        } else if uri.path().contains("metalink") {
            println!("METALINK: {uri}");
        } else {
            println!("!! UNKNOWN !! {uri}");
        }

        req.into()
    }

    async fn handle_response(&mut self, _ctx: &HttpContext, res: Response<Body>) -> Response<Body> {
        //println!("{:?}", res);
        res
    }
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