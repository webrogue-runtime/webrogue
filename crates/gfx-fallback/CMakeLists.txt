cmake_minimum_required(VERSION 3.22)

project(WEBROGUE_GFX_FALLBACK C)

set(BUILD_SHARED_LIBS FALSE)
set(SDL_SHARED FALSE)
set(SDL_STATIC TRUE)
add_subdirectory(SDL${WEBROGUE_GFX_SDL_VERSION})

add_library(
    wrgfxfallback STATIC 
    webrogue_gfx_ffi_sdl.c
    webrogue_gfx_ffi_sdl_events.c
)

target_include_directories(
    wrgfxfallback PRIVATE 
    SDL${WEBROGUE_GFX_SDL_VERSION}/include
)
target_compile_definitions(
    wrgfxfallback PRIVATE
    WEBROGUE_GFX_SDL_VERSION=${WEBROGUE_GFX_SDL_VERSION}
)
set(CMAKE_INSTALL_LIBDIR "")
install(TARGETS wrgfxfallback SDL${WEBROGUE_GFX_SDL_VERSION}-static)
