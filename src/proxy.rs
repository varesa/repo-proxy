use std::net::SocketAddr;
use hudsucker::async_trait::async_trait;
use hudsucker::certificate_authority::RcgenAuthority;
use hudsucker::{HttpContext, HttpHandler, NoopHandler, RequestOrResponse};
use hudsucker::hyper::client::HttpConnector;
use hudsucker::hyper::{Body, http, Method, Request, Response};
use hyper_rustls::HttpsConnector;
use crate::ca::Ca;
use crate::prefix_match::PrefixMatcher;
use crate::request_meta::{RequestMetadata, RequestMetadataBuidler, RequestType};

static mut PREFIX_MATCHER: Option<PrefixMatcher> = None;

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to install CTRL+C signal handler");
}



#[derive(Clone, Default, Debug)]
struct Handler {
    request_metadata: Option<RequestMetadata>,
}

#[async_trait]
impl HttpHandler for Handler {
    async fn handle_request(
        &mut self,
        ctx: &HttpContext,
        req: Request<Body>,
    ) -> RequestOrResponse {
        let mut metabuilder = RequestMetadataBuidler::new();

        if req.method() == Method::CONNECT {
            return req.into(); // We are not interested in the TCP proxy layer -> ignore
        }

        let view_name = unsafe {
            if let Ok(view) = PREFIX_MATCHER.as_ref().expect("prefix matcher not initialized").lookup(ctx.client_addr.ip()) {
                view
            } else {
                return RequestOrResponse::Response(http::Response::builder().status(403).body(Body::empty()).unwrap())
            }
        };
        metabuilder.view_name(view_name.clone());

        print!("[{view_name}] ");

        let uri = req.uri();
        metabuilder.uri(uri.clone());
        if uri.path().ends_with(".rpm") {
            println!("PACKAGE: {uri}");
            metabuilder.request_type(RequestType::Package);
        } else if uri.path().contains("repodata") {
            println!("REPODATA: {uri}");
            metabuilder.request_type(RequestType::Repodata);
        } else if uri.path().contains("metalink") {
            println!("METALINK: {uri}");
            metabuilder.request_type(RequestType::Metalink);
        } else {
            println!("!! UNKNOWN !! {uri}");
            metabuilder.request_type(RequestType::Unknown);
        }

        self.request_metadata = Some(metabuilder.build());
        req.into()
    }

    async fn handle_response(&mut self, _ctx: &HttpContext, res: Response<Body>) -> Response<Body> {
        let _request_meta = self.request_metadata.as_ref().expect("No request metadata set for response");
        res
    }
}

pub struct Proxy  {
    hudsucker: hudsucker::Proxy<HttpsConnector<HttpConnector>, RcgenAuthority, Handler, NoopHandler>
}

impl Proxy {
    pub fn try_new(ca: Ca, prefix_matcher: PrefixMatcher) -> Result<Self, anyhow::Error> {
        let rcgen = RcgenAuthority::new(ca.private_key, ca.certificate, 1_000)?;
        let proxy = hudsucker::Proxy::builder()
            .with_addr(SocketAddr::from(([0,0,0,0], 8777)))
            .with_rustls_client()
            .with_ca(rcgen)
            .with_http_handler(Handler::default())
            .build();

        unsafe {
            PREFIX_MATCHER = Some(prefix_matcher);
        }

        Ok(Self {
            hudsucker: proxy,
        })
    }

    pub async fn start(self) -> Result<(), anyhow::Error> {
        self.hudsucker.start(shutdown_signal()).await?;
        Ok(())
    }
}