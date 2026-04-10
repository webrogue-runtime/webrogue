use std::{net::SocketAddr, sync::Arc};

use async_trait::async_trait;
use axum_extra::extract::CookieJar;
use headers::Host;
use http::Method;
use tokio::sync::Mutex;
use tower_http::cors::{Any, CorsLayer};
use webrogue_launcher_server_openapi::{
    apis::default::{ConnectDeviceResponse, GetDeviceNameResponse},
    models,
};

use crate::{debug_connection::DebugConnection, stored_config::get_stored_config, LauncherConfig};

struct Inner {
    pub connection: Option<DebugConnection>,
    pub launcher_config: Arc<dyn LauncherConfig>,
}

impl Inner {
    fn new(launcher_config: Arc<dyn LauncherConfig>) -> Self {
        Self {
            connection: None,
            launcher_config,
        }
    }
}

struct Api {
    pub(crate) inner: Arc<Mutex<Inner>>,
}

#[async_trait]
impl webrogue_launcher_server_openapi::apis::default::Default<anyhow::Error> for Api {
    async fn get_device_name(
        &self,
        _method: &Method,
        _host: &Host,
        _cookies: &CookieJar,
    ) -> anyhow::Result<GetDeviceNameResponse> {
        let inner = self.inner.lock().await;
        let config = get_stored_config(&inner.launcher_config.storage_path())?;
        Ok(GetDeviceNameResponse::Status200_Success(
            models::GetDeviceNameResponse {
                name: config.device_name,
            },
        ))
    }

    async fn connect_device(
        &self,
        _method: &Method,
        _host: &Host,
        _cookies: &CookieJar,
        body: &models::ConnectDeviceRequest,
    ) -> anyhow::Result<ConnectDeviceResponse> {
        let mut inner = self.inner.lock().await;
        let stored_config = get_stored_config(&inner.launcher_config.storage_path())?;
        inner.connection = Some(DebugConnection::new(
            body.auth_token.clone(),
            stored_config.device_name,
            inner.launcher_config.clone(),
        ));
        Ok(ConnectDeviceResponse::Status200_Success)
    }
}

#[async_trait]
impl webrogue_launcher_server_openapi::apis::ErrorHandler<anyhow::Error> for Api {
    async fn handle_error(
        &self,
        _method: &::http::Method,
        _host: &headers::Host,
        _cookies: &axum_extra::extract::CookieJar,
        error: anyhow::Error,
    ) -> Result<axum::response::Response, http::StatusCode> {
        axum::response::Response::builder()
            .status(http::StatusCode::INTERNAL_SERVER_ERROR)
            .body(axum::body::Body::from(error.to_string()))
            .map_err(|_| http::StatusCode::INTERNAL_SERVER_ERROR)
    }
}

pub fn run_server(launcher_config: Arc<dyn LauncherConfig>) {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    runtime.block_on(async {
        let addr = SocketAddr::from(([127, 0, 0, 1], 0));

        let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let router = make_router(launcher_config).await.unwrap();
        println!("webrogue-vscode-server is listening on port {}", port);
        axum::serve(listener, router).await.unwrap()
    });
}

pub async fn make_router(launcher_config: Arc<dyn LauncherConfig>) -> anyhow::Result<axum::Router> {
    Ok(webrogue_launcher_server_openapi::server::new(Arc::new(Api {
        inner: Arc::new(Mutex::new(Inner::new(launcher_config))),
    }))
    .layer(
        CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any),
    ))
}
