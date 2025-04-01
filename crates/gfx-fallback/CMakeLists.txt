project(WEBROGUE_GFX_FALLBACK C)

set(SDL_SHARED FALSE)
set(SDL_STATIC TRUE)
add_subdirectory(SDL)

add_library(
    wrgfxfallback STATIC 
    webrogue_gfx_ffi_sdl.c
    webrogue_gfx_ffi_sdl_events.c
)

target_include_directories(
    wrgfxfallback PRIVATE 
    SDL/include
)
target_compile_definitions(
    wrgfxfallback PRIVATE
    WEBROGUE_GFX_SDL_VERSION=2
)
set(CMAKE_INSTALL_LIBDIR "")
install(TARGETS wrgfxfallback SDL2-static)
