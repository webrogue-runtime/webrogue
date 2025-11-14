use std::{net::SocketAddr, path::PathBuf, sync::Arc};

use async_trait::async_trait;
use axum_extra::extract::{CookieJar, Host};
use http::Method;
use tokio::sync::Mutex;
use tower_http::cors::{Any, CorsLayer};
use webrogue_launcher_server_openapi::{
    apis::default::{GetWrappConfigResponse, MakePeerConnectionResponse},
    models::{Sdp, WebrogueConfig},
};
use webrogue_wrapp::RealVFSBuilder;

use crate::debug_connection::IncomingDebugConnection;

pub trait ServerConfig: Send + Sync {
    fn storage_path(&self) -> PathBuf;
    fn run(&self, vfs_builder: RealVFSBuilder) -> anyhow::Result<()>;
}

struct Inner {
    pub connection: Option<IncomingDebugConnection>,
    pub config: Arc<dyn ServerConfig>,
}

impl Inner {
    fn new(config: Arc<dyn ServerConfig>) -> Self {
        Self {
            connection: None,
            config,
        }
    }

    async fn make_peer_connection(&mut self, offer: &Sdp) -> anyhow::Result<Sdp> {
        if let Some(connection) = &self.connection {
            connection.close().await;
        }
        let connection = IncomingDebugConnection::new(&offer.sdp, self.config.clone()).await?;
        let answer = connection.answer.clone();
        self.connection = Some(connection);
        Ok(Sdp::new(answer))
    }
}

struct Api {
    pub(crate) inner: Arc<Mutex<Inner>>,
}

#[async_trait]
impl webrogue_launcher_server_openapi::apis::default::Default<anyhow::Error> for Api {
    async fn get_wrapp_config(
        &self,
        _method: &Method,
        _host: &Host,
        _cookies: &CookieJar,
    ) -> anyhow::Result<GetWrappConfigResponse, anyhow::Error> {
        return Ok(
            GetWrappConfigResponse::Status200_SuccessfullyRetrievedDeviceList(WebrogueConfig::new(
                "name".to_owned(),
                "id".to_owned(),
                "version".to_owned(),
            )),
        );
    }

    async fn make_peer_connection(
        &self,
        _method: &Method,
        _host: &Host,
        _cookies: &CookieJar,
        body: &Sdp,
    ) -> Result<MakePeerConnectionResponse, anyhow::Error> {
        let result = self
            .inner
            .lock()
            .await
            .make_peer_connection(body)
            .await
            .map_err(|e| e)?;

        return Ok(MakePeerConnectionResponse::Status200_SuccessfullyMadePeerConnection(result));
    }
}

#[async_trait]
impl webrogue_launcher_server_openapi::apis::ErrorHandler<anyhow::Error> for Api {
    async fn handle_error(
        &self,
        _method: &::http::Method,
        _host: &axum_extra::extract::Host,
        _cookies: &axum_extra::extract::CookieJar,
        error: anyhow::Error,
    ) -> Result<axum::response::Response, http::StatusCode> {
        println!("{}", error);
        axum::response::Response::builder()
            .status(http::StatusCode::INTERNAL_SERVER_ERROR)
            .body(axum::body::Body::from(error.to_string()))
            .map_err(|_| http::StatusCode::INTERNAL_SERVER_ERROR)
    }
}

pub fn run_server(config: Arc<dyn ServerConfig>) {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    runtime.block_on(async {
        let addr = SocketAddr::from(([127, 0, 0, 1], 0));

        let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let router = make_router(config).await.unwrap();
        println!("webrogue-vscode-server is listening on port {}", port);
        axum::serve(listener, router).await.unwrap()
    });
}

pub async fn make_router(config: Arc<dyn ServerConfig>) -> anyhow::Result<axum::Router> {
    Ok(webrogue_launcher_server_openapi::server::new(Arc::new(Api {
        inner: Arc::new(Mutex::new(Inner::new(config))),
    }))
    .layer(
        CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any),
    ))
}
