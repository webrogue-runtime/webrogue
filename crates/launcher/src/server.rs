use std::{net::SocketAddr, sync::Arc};

use async_trait::async_trait;
use axum_extra::extract::{CookieJar, Host};
use http::Method;
use tower_http::cors::{Any, CorsLayer};
use webrogue_launcher_server_openapi::{
    apis::default::GetWrappConfigResponse, models::WebrogueConfig,
};

struct Api {}

impl webrogue_launcher_server_openapi::apis::ErrorHandler for Api {}

#[async_trait]
impl webrogue_launcher_server_openapi::apis::default::Default for Api {
    async fn get_wrapp_config(
        &self,
        _method: &Method,
        _host: &Host,
        _cookies: &CookieJar,
    ) -> anyhow::Result<GetWrappConfigResponse, ()> {
        return Ok(
            GetWrappConfigResponse::Status200_SuccessfullyRetrievedDeviceList(WebrogueConfig::new(
                "name".to_owned(),
                "id".to_owned(),
                "version".to_owned(),
            )),
        );
    }
}

pub fn run_server() {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            let addr = SocketAddr::from(([127, 0, 0, 1], 0));

            let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
            let port = listener.local_addr().unwrap().port();
            println!("webrogue-vscode-server is listening on port {}", port);
            let router = make_router();
            axum::serve(listener, router).await.unwrap()
        });
}

pub fn make_router() -> axum::Router {
    webrogue_launcher_server_openapi::server::new(Arc::new(Api {})).layer(
        CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any),
    )
}
