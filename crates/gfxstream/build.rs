fn main() {
    cfg_aliases::cfg_aliases! {
        signal_based_shadow_blob: { all(not(target_os = "macos"), not(target_os = "ios")) },
    }
}
