use webrogue_hub_client::{HTTP_BASE_ADDR, WS_BASE_ADDR};

fn localhost_api_addr_port() -> &'static str {
    if cfg!(target_os = "android") {
        "10.0.2.2:8080"
    } else {
        "localhost:8080"
    }
}

fn use_localhost_api() -> bool {
    // cfg!(debug_assertions)
    false
}

fn use_localhost_ui() -> bool {
    // cfg!(debug_assertions)
    false
}

pub fn http_api_url() -> String {
    if use_localhost_api() {
        format!("http://{}", localhost_api_addr_port())
    } else {
        HTTP_BASE_ADDR.to_owned()
    }
}

pub fn ws_api_url() -> String {
    if use_localhost_api() {
        format!("ws://{}", localhost_api_addr_port())
    } else {
        WS_BASE_ADDR.to_owned()
    }
}

pub fn assets_url() -> &'static str {
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
