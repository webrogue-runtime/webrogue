fn main() {
    cfg_aliases::cfg_aliases! {
        signal_based_shadow_blob: { not(target_arch = "wasm") },
    }
}
