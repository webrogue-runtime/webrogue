fn main() {
    cfg_aliases::cfg_aliases! {
        signal_bases_shadow_blob: { all(not(target_os = "macos"), not(target_os = "ios")) },
    }
}
