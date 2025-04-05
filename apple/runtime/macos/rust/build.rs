fn main() {
    #[cfg(target_os = "macos")]
    cc::Build::new()
        .file("../../../../crates/gfx-fallback/webrogue_gfx_ffi_sdl.c")
        .file("../../../../crates/gfx-fallback/webrogue_gfx_ffi_sdl_events.c")
        .define("WEBROGUE_GFX_SDL_VERSION", "2")
        .include("../../external/SDL2/include")
        .compile("webrogue_gfx_ffi_sdl2");
}
