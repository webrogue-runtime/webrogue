fn main() {
    cc::Build::new()
        .file("../../../../crates/gfx-fallback/webrogue_gfx_ffi_sdl2.c")
        .include("../../../external/SDL2/include")
        .compile("webrogue_gfx_ffi_sdl2");
}
