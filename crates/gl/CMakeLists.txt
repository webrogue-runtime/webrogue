
project(WEBROGUE_GL)

# set(AEMU_COMMON_BUILD_CONFIG gfxstream)
# set(ENABLE_VKCEREAL_TESTS OFF)
# add_subdirectory(../../external/aemu aemu)

# add_library(flatbuffers INTERFACE)
# add_library(gfxstream_egl_headers INTERFACE)
# add_library(libdrm_headers INTERFACE)
# add_subdirectory(../../external/gfxstream gfxstream)
add_library(webrogue_gl dummy.c)

install(TARGETS webrogue_gl)
