fn main() {
    #[cfg(target_os = "macos")]
    cc::Build::new()
        .file("../../../../crates/gfx-fallback/webrogue_gfx_ffi_sdl.c")
        .file("../../../../crates/gfx-fallback/webrogue_gfx_ffi_sdl_events.c")
        .define("WEBROGUE_GFX_SDL_VERSION", "3")
        .include("../../external/SDL3/include")
        .include("../../external/SDL3/include/SDL3")
        .compile("webrogue_gfx_ffi_sdl3");
}
