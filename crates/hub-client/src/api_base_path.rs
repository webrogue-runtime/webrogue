pub static STAGE_WS_BASE_ADDR: &str = "wss://stage.webrogue.dev";
pub static STAGE_HTTP_BASE_ADDR: &str = "https://stage.webrogue.dev";

fn localhost_api_addr_port() -> &'static str {
    if cfg!(target_os = "android") {
        "10.0.2.2:8080"
    } else {
        "localhost:8080"
    }
}

fn use_localhost_api() -> bool {
    cfg!(debug_assertions)
    // false
}

fn use_localhost_ui() -> bool {
    cfg!(debug_assertions)
    // false
}

pub fn http_api_url() -> String {
    if use_localhost_api() {
        format!("http://{}", localhost_api_addr_port())
    } else {
        STAGE_HTTP_BASE_ADDR.to_owned()
    }
}

pub fn ws_api_url() -> String {
    if use_localhost_api() {
        format!("ws://{}", localhost_api_addr_port())
    } else {
        STAGE_WS_BASE_ADDR.to_owned()
    }
}

pub fn assets_url() -> &'static str {
    if use_localhost_ui() {
        if cfg!(target_os = "android") {
            "http://10.0.2.2:5173/"
        } else {
            "http://localhost:5173/"
        }
    } else {
        "wrlauncher://webrogue.dev/"
    }
}
