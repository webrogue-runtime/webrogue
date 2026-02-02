use std::sync::Arc;

use bytes::Bytes;
use futures_util::stream::TryStreamExt;
use http::{Method, Request, Uri};
use lazy_static::lazy_static;
use tokio_stream::{self, StreamExt as _};
use tower_service::Service;
use wry::{
    http, raw_window_handle::HasWindowHandle, RequestAsyncResponder, WebView, WebViewBuilder,
    WebViewId,
};

use crate::{
    mailbox::Mailbox,
    server::{make_router, ServerConfig},
    MailboxInternal,
};

lazy_static! {
    static ref RUNTIME: tokio::runtime::Runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
}

fn use_localhost_ui() -> bool {
    cfg!(debug_assertions)
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

pub fn build_webview<W: HasWindowHandle, MailboxImpl: Mailbox + 'static>(
    window: &W,
    as_child: bool,
    server_config: Arc<dyn ServerConfig>,
    mailbox_factory: impl FnOnce(MailboxInternal) -> MailboxImpl,
) -> Result<(WebView, MailboxImpl), wry::Error> {
    let router = RUNTIME.block_on(async { make_router(server_config).await.unwrap() });
    let router1 = router.clone();

    let mailbox_internal = MailboxInternal::new();
    let mailbox_impl = (mailbox_factory)(mailbox_internal.clone());
    let mailbox_1 = (mailbox_internal, mailbox_impl.clone());
    let mut builder = WebViewBuilder::new()
        .with_url(assets_url())
        .with_devtools(cfg!(debug_assertions))
        .with_asynchronous_custom_protocol("wrlauncher".into(), make_handler)
        .with_initialization_script("onWRLauncherMessage = undefined")
        .with_ipc_handler(move |request| {
            let mailbox_2 = mailbox_1.clone();
            let mut local_router = router1.clone();
            RUNTIME.spawn(async move {
                #[derive(serde::Serialize)]
                struct IPCResponse {
                    id: u64,
                    status: u64,
                    body: Option<String>,
                }

                #[derive(serde::Deserialize)]
                struct IPCRequest {
                    id: u64,
                    path: String,
                    method: String,
                    body: Option<String>,
                    headers: Vec<(String, String)>,
                }
                let Ok(request) = serde_json::from_str::<IPCRequest>(&request.into_body()) else {
                    return;
                };

                let response: anyhow::Result<IPCResponse> = async {
                    let body = request
                        .body
                        .unwrap_or_else(|| "".to_string())
                        .as_bytes()
                        .to_vec();
                    let method = Method::from_bytes(request.method.as_bytes())?;
                    let uri = Uri::builder()
                        .scheme("wrlauncher")
                        .authority("api")
                        .path_and_query(request.path)
                        .build()?;

                    let stream: tokio_stream::Iter<
                        std::vec::IntoIter<Result<Bytes, anyhow::Error>>,
                    > = tokio_stream::iter(vec![Ok(Bytes::from(body))]);
                    let mut mapped_request_builder = Request::builder().method(method).uri(uri);
                    for (header_name, header_value) in request.headers {
                        mapped_request_builder =
                            mapped_request_builder.header(header_name, header_value);
                    }
                    let mapped_request = mapped_request_builder
                        .body(axum::body::Body::from_stream(stream))
                        .map_err(|e| anyhow::anyhow!("Failed to build request: {}", e))?;

                    let response = local_router
                        .call(mapped_request)
                        .await
                        .map_err(|e| anyhow::anyhow!("Router call failed: {}", e))?;
                    let status = response.status();
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

                    Ok(IPCResponse {
                        id: request.id,
                        status: status.as_u16() as u64,
                        body: Some(String::from_utf8(body_vec)?),
                    })
                }
                .await;

                let response = match response {
                    Ok(response) => response,
                    Err(e) => IPCResponse {
                        id: request.id,
                        status: 500,
                        body: Some(e.to_string()),
                    },
                };
                let Ok(response) = serde_json::to_string(&response) else {
                    return;
                };
                crate::mailbox::execute(mailbox_2.1, mailbox_2.0, |webview| {
                    let _ = webview.evaluate_script(&format!("onWRLauncherMessage({})", response));
                })
            });
        });

    if let Some(url) = api_url() {
        builder = builder.with_initialization_script(format!(
            "wrApiBasePath = \"{}\"",
            json_escape::escape_str(url)
        ))
    }

    let webview = if as_child {
        builder.build_as_child(window)?
    } else {
        builder.build(window)?
    };
    Ok((webview, mailbox_impl))
}

fn make_handler(
    _webview_id: WebViewId,
    request: Request<Vec<u8>>,
    responder: RequestAsyncResponder,
) {
    let authority = request.uri().authority().map(|a| a.as_str());
    match authority {
        Some("asset") | Some("asset.wrlauncher") => match get_asset_response(request) {
            Ok(r) => responder.respond(r),
            Err(e) => responder.respond(internal_server_error_response(&e.to_string())),
        },
        _ => responder.respond(not_found_response()),
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
