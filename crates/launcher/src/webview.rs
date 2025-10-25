use wry::{
    http,
    raw_window_handle::HasWindowHandle,
    WebView, WebViewBuilder,
};

pub fn build_webview<W: HasWindowHandle>(
    window: &W,
    as_child: bool,
) -> Result<WebView, wry::Error> {
    let builder = WebViewBuilder::new()
        .with_url("wrlauncher://asset/")
        .with_devtools(true)
        .with_custom_protocol("wrlauncher".into(), move |_webview_id, request| {
            match get_wry_response(request) {
                Ok(r) => r.map(Into::into),
                Err(e) => http::Response::builder()
                    .header(http::header::CONTENT_TYPE, "text/plain")
                    .status(500)
                    .body(e.to_string().as_bytes().to_vec())
                    .unwrap()
                    .map(Into::into),
            }
        });

    if as_child {
        builder.build_as_child(window)
    } else {
        builder.build(window)
    }
}

fn get_wry_response(
    request: http::Request<Vec<u8>>,
) -> Result<http::Response<&'static [u8]>, Box<dyn std::error::Error>> {
    if !matches!(request.uri().authority().map(|a| a.as_str()), Some("asset")) {
        return Ok(not_found_response());
    }

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
