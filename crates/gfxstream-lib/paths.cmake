set(GFX_SRC_DIR ${WEBROGUE_GFXSTREAM_DIR}/../../external/gfxstream)
set(AEMU_SRC_DIR ${WEBROGUE_GFXSTREAM_DIR}/../../external/aemu)

set(
    WEBROGUE_GFXSTREAM_SOURCES

    "${WEBROGUE_GFXSTREAM_DIR}/webrogue_gfxstream.cpp"
    
    "${GFX_SRC_DIR}/host/vulkan/VkDecoder.cpp"
    "${GFX_SRC_DIR}/host/vulkan/VulkanStream.cpp"
    "${GFX_SRC_DIR}/host/vulkan/VulkanHandleMapping.cpp"
    "${GFX_SRC_DIR}/host/vulkan/VulkanBoxedHandles.cpp"
    "${GFX_SRC_DIR}/host/vulkan/VkDecoderGlobalState.cpp"
    "${GFX_SRC_DIR}/host/vulkan/RenderThreadInfoVk.cpp"
    "${GFX_SRC_DIR}/host/vulkan/DebugUtilsHelper.cpp"
    "${GFX_SRC_DIR}/host/vulkan/DeviceOpTracker.cpp"
    "${GFX_SRC_DIR}/host/vulkan/VkCommonOperations.cpp"
    "${GFX_SRC_DIR}/host/vulkan/DeviceLostHelper.cpp"
    "${GFX_SRC_DIR}/host/vulkan/VkEmulatedPhysicalDeviceQueue.cpp"
    "${GFX_SRC_DIR}/host/vulkan/VkEmulatedPhysicalDeviceMemory.cpp"
    "${GFX_SRC_DIR}/host/vulkan/VkDecoderSnapshot.cpp"
    "${GFX_SRC_DIR}/host/vulkan/VkReconstruction.cpp"
    "${GFX_SRC_DIR}/host/vulkan/SwapChainStateVk.cpp"
    "${GFX_SRC_DIR}/host/vulkan/DependencyGraph.cpp"
    "${GFX_SRC_DIR}/host/vulkan/VkUtils.cpp"
    "${GFX_SRC_DIR}/host/vulkan/cereal/common/goldfish_vk_dispatch.cpp"
    "${GFX_SRC_DIR}/host/vulkan/cereal/common/goldfish_vk_transform.cpp"
    "${GFX_SRC_DIR}/host/vulkan/cereal/common/goldfish_vk_extension_structs.cpp"
    "${GFX_SRC_DIR}/host/vulkan/cereal/common/goldfish_vk_reserved_marshaling.cpp"
    "${GFX_SRC_DIR}/host/vulkan/cereal/common/goldfish_vk_deepcopy.cpp"
    "${GFX_SRC_DIR}/host/vulkan/cereal/common/goldfish_vk_marshaling.cpp"
    "${GFX_SRC_DIR}/host/vulkan/emulated_textures/GpuDecompressionPipeline.cpp"
    "${GFX_SRC_DIR}/host/vulkan/emulated_textures/AstcTexture.cpp"
    "${GFX_SRC_DIR}/host/vulkan/emulated_textures/CompressedImageInfo.cpp"
    "${GFX_SRC_DIR}/host/compressed_textures/AstcCpuDecompressorNoOp.cpp"
    "${GFX_SRC_DIR}/host/features/Features.cpp"
    "${GFX_SRC_DIR}/host/backend/external_object_manager.cpp"
    "${GFX_SRC_DIR}/host/backend/vm_operations.cpp"
    "${GFX_SRC_DIR}/host/backend/graphics_driver_lock.cpp"
    "${GFX_SRC_DIR}/host/backend/stream_utils.cpp"
    "${GFX_SRC_DIR}/host/health/HealthMonitor.cpp"
    "${GFX_SRC_DIR}/host/health/TestClock.cpp"
    "${GFX_SRC_DIR}/host/metrics/Metrics.cpp"
    "${GFX_SRC_DIR}/common/base/UdmabufCreator_stub.cpp"
    "${GFX_SRC_DIR}/common/base/System.cpp"
    "${GFX_SRC_DIR}/common/logging/logging.cpp"
)

if(WIN32)
  list(APPEND WEBROGUE_GFXSTREAM_SOURCES "${GFX_SRC_DIR}/common/base/Thread_win32.cpp")
else()
  list(APPEND WEBROGUE_GFXSTREAM_SOURCES "${GFX_SRC_DIR}/common/base/Thread_pthread.cpp")
endif()

set(
    WEBROGUE_GFXSTREAM_INCLUDE_DIRS

    "${GFX_SRC_DIR}/host"
    "${GFX_SRC_DIR}/host/vulkan"
    "${GFX_SRC_DIR}/host/vulkan/cereal"
    "${GFX_SRC_DIR}/host/vulkan/cereal/common"
    "${GFX_SRC_DIR}/host/features/include"
    "${GFX_SRC_DIR}/host/gl/gl-host-common/include"
    "${GFX_SRC_DIR}/host/features/include/gfxstream/host"
    "${GFX_SRC_DIR}/host/tracing/include"
    "${GFX_SRC_DIR}/host/backend/include"
    "${GFX_SRC_DIR}/common/vulkan/include"
    "${GFX_SRC_DIR}/common/utils/include"
    "${GFX_SRC_DIR}/include"
    "${GFX_SRC_DIR}/utils/include"
    "${GFX_SRC_DIR}/third-party/renderdoc/include"
    "${GFX_SRC_DIR}/third-party/glm/include"
    "${GFX_SRC_DIR}/host/compressed_textures/include"
    "${GFX_SRC_DIR}/host/health/include"
    "${GFX_SRC_DIR}/common/base/include"
    "${GFX_SRC_DIR}/host/metrics/include"
    "${GFX_SRC_DIR}/common/logging/include"
    "${GFX_SRC_DIR}/host/decoder_common/include"
    "${GFX_SRC_DIR}/third_party/vulkan/include"
    "${GFX_SRC_DIR}/host/include"
    "${GFX_SRC_DIR}/host/iostream/include"
    "${GFX_SRC_DIR}/third_party/glm/include"
    "${GFX_SRC_DIR}/third_party/glm/include"
    "${GFX_SRC_DIR}/host/renderdoc/include"
    "${GFX_SRC_DIR}/third_party/renderdoc/include"
    "${GFX_SRC_DIR}/host/library/include"
    "${GFX_SRC_DIR}/host/snapshot/include"
    "${GFX_SRC_DIR}/third_party/astc-encoder/Source"
    "${GFX_SRC_DIR}/third_party/opengl/include"
)
