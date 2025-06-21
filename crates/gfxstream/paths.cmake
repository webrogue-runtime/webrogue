set(GFX_SRC_DIR ${WEBROGUE_GFXSTREAM_DIR}/../../external/gfxstream)
set(AEMU_SRC_DIR ${WEBROGUE_GFXSTREAM_DIR}/../../external/aemu)

set(
    WEBROGUE_GFXSTREAM_SOURCES

    ${WEBROGUE_GFXSTREAM_DIR}/webrogue_gfxstream.cpp

    ${GFX_SRC_DIR}/host/vulkan/VkDecoder.cpp
    ${GFX_SRC_DIR}/host/vulkan/VulkanStream.cpp
    ${GFX_SRC_DIR}/host/vulkan/VulkanHandleMapping.cpp
    ${GFX_SRC_DIR}/host/vulkan/VulkanBoxedHandles.cpp
    ${GFX_SRC_DIR}/host/vulkan/VkDecoderGlobalState.cpp
    ${GFX_SRC_DIR}/host/vulkan/DebugUtilsHelper.cpp
    ${GFX_SRC_DIR}/host/vulkan/DeviceOpTracker.cpp
    ${GFX_SRC_DIR}/host/vulkan/VkCommonOperations.cpp
    ${GFX_SRC_DIR}/host/vulkan/vk_util.cpp
    ${GFX_SRC_DIR}/host/vulkan/DeviceLostHelper.cpp
    ${GFX_SRC_DIR}/host/vulkan/VkEmulatedPhysicalDeviceQueue.cpp
    ${GFX_SRC_DIR}/host/vulkan/VkEmulatedPhysicalDeviceMemory.cpp
    
    ${GFX_SRC_DIR}/host/vulkan/cereal/common/goldfish_vk_dispatch.cpp
    ${GFX_SRC_DIR}/host/vulkan/emulated_textures/GpuDecompressionPipeline.cpp
    ${GFX_SRC_DIR}/host/vulkan/emulated_textures/AstcTexture.cpp
    ${GFX_SRC_DIR}/host/vulkan/emulated_textures/CompressedImageInfo.cpp
    ${GFX_SRC_DIR}/host/compressedTextureFormats/AstcCpuDecompressorNoOp.cpp # TODO impl
    

    ${AEMU_SRC_DIR}/host-common/GfxstreamFatalError.cpp
    ${AEMU_SRC_DIR}/base/Tracing.cpp
    ${AEMU_SRC_DIR}/base/Stream.cpp
    ${AEMU_SRC_DIR}/base/Metrics.cpp
)

set(
    WEBROGUE_GFXSTREAM_INCLUDE_DIRS

    ${GFX_SRC_DIR}/host
    ${GFX_SRC_DIR}/host/vulkan
    ${GFX_SRC_DIR}/host/vulkan/cereal
    ${GFX_SRC_DIR}/host/vulkan/cereal/common
    ${GFX_SRC_DIR}/host/features/include
    ${GFX_SRC_DIR}/host/gl/gl-host-common/include
    ${GFX_SRC_DIR}/host/features/include/gfxstream/host
    ${GFX_SRC_DIR}/host/tracing/include
    ${GFX_SRC_DIR}/host/backend/include
    ${GFX_SRC_DIR}/common/vulkan/include
    ${GFX_SRC_DIR}/common/utils/include
    ${GFX_SRC_DIR}/include
    ${GFX_SRC_DIR}/utils/include

    ${GFX_SRC_DIR}/third-party/renderdoc/include
    ${GFX_SRC_DIR}/third-party/glm/include
    # ${GFX_SRC_DIR}/third-party/astc-encoder/Source

    ${AEMU_SRC_DIR}/snapshot/include
    ${AEMU_SRC_DIR}/base/include
    ${AEMU_SRC_DIR}/host-common/include
)
