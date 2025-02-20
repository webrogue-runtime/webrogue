project(WEBROGUE_GFX_FALLBACK C CXX)

set(GFX_SRC_DIR ${CMAKE_CURRENT_LIST_DIR}/../../external/gfxstream)
set(AEMU_SRC_DIR ${CMAKE_CURRENT_LIST_DIR}/../../external/aemu)

add_library(
    wrgfxfallback STATIC 
    webrogue_gfx_ffi_sdl2.cpp
    ${AEMU_SRC_DIR}/host-common/vm_operations.cpp
    ${AEMU_SRC_DIR}/host-common/crash_reporter.cpp
)

set(DEPENDENCY_RESOLUTION "WEBROGUE" CACHE STRING "" FORCE)
set(DEPENDENCY_RESOLUTION "WEBROGUE")
add_subdirectory(${AEMU_SRC_DIR} aemu EXCLUDE_FROM_ALL)
add_subdirectory(${GFX_SRC_DIR} gfxstream EXCLUDE_FROM_ALL)
target_link_libraries(wrgfxfallback aemu-base logging-base gles2_dec GLSnapshot apigen-codec-common)

set(SDL_SHARED FALSE)
set(SDL_STATIC TRUE)
add_subdirectory(SDL)
target_include_directories(
    wrgfxfallback PRIVATE 
    SDL/include
    ${GFX_SRC_DIR}/host
)
set(CMAKE_INSTALL_LIBDIR "")
install(TARGETS wrgfxfallback SDL2-static aemu-base logging-base gles2_dec GLSnapshot apigen-codec-common)
