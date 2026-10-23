mod compose;

pub use compose::compose;

cfg_select! {
    feature = "wasm-opt" => {
        mod optimize;
        pub use optimize::*;
    }
    _ => {
        mod optimize_stub;
        pub use optimize_stub::*;
    }
}
