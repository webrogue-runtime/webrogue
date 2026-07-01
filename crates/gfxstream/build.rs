fn main() {
    cfg_aliases::cfg_aliases! {
        shadow_blob: { all(not(target_os = "macos"), not(target_os = "ios")) },
        signal_based_shadow_blob: { all(shadow_blob, not(target_arch = "wasm")) },
    }
}
