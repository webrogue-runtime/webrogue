project(WEBROGUE_GFXSTREAM CXX)

set(GFX_SRC_DIR ${CMAKE_CURRENT_LIST_DIR}/../../external/gfxstream)
set(AEMU_SRC_DIR ${CMAKE_CURRENT_LIST_DIR}/../../external/aemu)

add_library(
    webrogue_gfxstream STATIC
    webrogue_gfxstream.cpp
    ${GFX_SRC_DIR}/host/gl/glsnapshot/GLSnapshot.cpp
    ${GFX_SRC_DIR}/host/gl/gles2_dec/GLESv2Decoder.cpp
    ${GFX_SRC_DIR}/host/gl/gles2_dec/gles2_server_context.cpp
    ${GFX_SRC_DIR}/host/gl/gles2_dec/gles2_dec.cpp
    ${GFX_SRC_DIR}/host/apigen-codec-common/ChecksumCalculatorThreadInfo.cpp
    ${GFX_SRC_DIR}/host/apigen-codec-common/ChecksumCalculator.cpp
    ${AEMU_SRC_DIR}/host-common/vm_operations.cpp
    ${AEMU_SRC_DIR}/host-common/crash_reporter.cpp
    ${AEMU_SRC_DIR}/base/Tracing.cpp
    ${AEMU_SRC_DIR}/base/Stream.cpp
)

target_include_directories(
    webrogue_gfxstream PRIVATE
    ${GFX_SRC_DIR}/host
    ${GFX_SRC_DIR}/include
    ${GFX_SRC_DIR}/host/apigen-codec-common
    ${GFX_SRC_DIR}/host/gl/glsnapshot
    ${AEMU_SRC_DIR}/base/include
    ${AEMU_SRC_DIR}/host-common/include
)
set(CMAKE_INSTALL_LIBDIR "")
install(TARGETS webrogue_gfxstream)
