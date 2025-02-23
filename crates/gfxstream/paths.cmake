set(GFX_SRC_DIR ${WEBROGUE_GFXSTREAM_DIR}/../../external/gfxstream)
set(AEMU_SRC_DIR ${WEBROGUE_GFXSTREAM_DIR}/../../external/aemu)

set(
    WEBROGUE_GFXSTREAM_SOURCES
    ${WEBROGUE_GFXSTREAM_DIR}/webrogue_gfxstream.cpp
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

set(
    WEBROGUE_GFXSTREAM_INCLUDE_DIRS
    ${GFX_SRC_DIR}/host
    ${GFX_SRC_DIR}/include
    ${GFX_SRC_DIR}/host/apigen-codec-common
    ${GFX_SRC_DIR}/host/gl/glsnapshot
    ${AEMU_SRC_DIR}/base/include
    ${AEMU_SRC_DIR}/host-common/include
)
