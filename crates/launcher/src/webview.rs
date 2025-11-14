use std::sync::Arc;

use anyhow;
use bytes::Bytes;
use futures_util::stream::TryStreamExt;
use http::{Request, Response};
use lazy_static::lazy_static;
use tokio_stream::{self, StreamExt as _};
use tower_service::Service;
use wry::{
    http, raw_window_handle::HasWindowHandle, RequestAsyncResponder, WebView, WebViewBuilder,
    WebViewId,
};

use crate::server::{make_router, ServerConfig};

lazy_static! {
    static ref RUNTIME: tokio::runtime::Runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
}

fn use_localhost_ui() -> bool {
    cfg!(debug_assertions)
}

fn launcher_api() -> Option<&'static str> {
    None
    // cfg!(debug_assertions)
}

fn api_url() -> Option<&'static str> {
    if use_localhost_ui() {
        if cfg!(target_os = "android") {
            Some("http://10.0.2.2:8080")
        } else {
            Some("http://localhost:8080")
        }
    } else {
        None
    }
}

fn assets_url() -> &'static str {
    if use_localhost_ui() {
        if cfg!(target_os = "android") {
            "http://10.0.2.2:5202/"
        } else {
            "http://localhost:5202/"
        }
    } else {
        "wrlauncher://asset/"
    }
}

pub fn build_webview<W: HasWindowHandle>(
    window: &W,
    as_child: bool,
    server_config: Arc<dyn ServerConfig>,
) -> Result<WebView, wry::Error> {
    let router = RUNTIME.block_on(async { make_router(server_config).await.unwrap() });

    let mut builder = WebViewBuilder::new()
        .with_url(assets_url())
        .with_devtools(cfg!(debug_assertions))
        .with_asynchronous_custom_protocol("wrlauncher".into(), make_handler(router));

    if let Some(url) = api_url() {
        builder = builder.with_initialization_script(format!(
            "wrApiBasePath = \"{}\"",
            json_escape::escape_str(url)
        ))
    }
    builder = builder.with_initialization_script(format!(
        "wrLauncherHostChannelUrl = \"{}\"",
        json_escape::escape_str(launcher_api().unwrap_or("wrlauncher://api/"))
    ));

    if as_child {
        builder.build_as_child(window)
    } else {
        builder.build(window)
    }
}

fn make_handler(
    router: axum::Router,
) -> impl Fn(WebViewId, Request<Vec<u8>>, RequestAsyncResponder) {
    move |_webview_id, request: http::Request<Vec<u8>>, responder: wry::RequestAsyncResponder| {
        let authority = request.uri().authority().map(|a| a.as_str());
        match authority {
            Some("asset") => match get_asset_response(request) {
                Ok(r) => responder.respond(r),
                Err(e) => responder.respond(internal_server_error_response(&e.to_string())),
            },
            Some("api") => {
                let mut local_router = router.clone();
                let _ = RUNTIME.spawn(async move {
                    let result = async {
                        let method = request.method().clone();
                        let uri = request.uri().clone();
                        let headers = request.headers().clone();
                        let body = request.into_body();
                        let stream: tokio_stream::Iter<
                            std::vec::IntoIter<Result<Bytes, anyhow::Error>>,
                        > = tokio_stream::iter(vec![Ok(Bytes::from(body))]);
                        let mut mapped_request_builder = Request::builder().method(method).uri(uri);
                        for (header_name, header_value) in headers {
                            if let Some(header_name) = header_name {
                                mapped_request_builder =
                                    mapped_request_builder.header(header_name, header_value);
                            }
                        }
                        let mapped_request = mapped_request_builder
                            .body(axum::body::Body::from_stream(stream))
                            .map_err(|e| anyhow::anyhow!("Failed to build request: {}", e))?;

                        let response = local_router
                            .call(mapped_request)
                            .await
                            .map_err(|e| anyhow::anyhow!("Router call failed: {}", e))?;
                        let status = response.status();
                        let headers = response.headers().clone();
                        let body_stream = response.into_body().into_data_stream();
                        let body_vec: Vec<u8> = body_stream
                            .map(|result| {
                                result.map_err(|e| anyhow::anyhow!("Body stream error: {}", e))
                            })
                            .try_collect::<Vec<bytes::Bytes>>()
                            .await?
                            .into_iter()
                            .flat_map(|bytes| bytes.into_iter())
                            .collect();
                        let mut response_builder = Response::builder().status(status);
                        for (header_name, header_value) in headers {
                            if let Some(header_name) = header_name {
                                response_builder =
                                    response_builder.header(header_name, header_value);
                            }
                        }
                        response_builder
                            .body(body_vec)
                            .map_err(|e| anyhow::anyhow!("Failed to build response: {}", e))
                    }
                    .await;

                    match result {
                        Ok(response) => responder.respond(response),
                        Err(e) => responder.respond(internal_server_error_response(&e.to_string())),
                    }
                });
            }
            _ => responder.respond(not_found_response()),
        }
    }
}

fn get_asset_response(
    request: http::Request<Vec<u8>>,
) -> Result<http::Response<&'static [u8]>, Box<dyn std::error::Error>> {
    let path = request.uri().path();
    let data: Option<(&[u8], _)> = match path {
        "/" => Some((include_bytes!("../webview_assets/index.html"), "text/html")),
        "/main.js" => Some((
            include_bytes!("../webview_assets/main.js"),
            "text/javascript",
        )),
        "/style.css" => Some((include_bytes!("../webview_assets/style.css"), "text/css")),
        _ => None,
    };
    let Some((content, mimetype)) = data else {
        return Ok(not_found_response());
    };

    http::Response::builder()
        .header(http::header::CONTENT_TYPE, mimetype)
        .body(content)
        .map_err(Into::into)
}

fn not_found_response() -> http::Response<&'static [u8]> {
    http::Response::builder()
        .header(http::header::CONTENT_TYPE, "text/plain")
        .status(404)
        .body(b"not found" as &[u8])
        .unwrap()
}

fn internal_server_error_response(message: &str) -> http::Response<Vec<u8>> {
    http::Response::builder()
        .header(http::header::CONTENT_TYPE, "text/plain")
        .status(500)
        .body(message.as_bytes().to_vec())
        .unwrap()
}
