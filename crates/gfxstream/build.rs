use std::str::FromStr as _;

fn main() {
    let _crate_manifest_dir =
        std::path::PathBuf::from_str(&std::env::var("CARGO_MANIFEST_DIR").unwrap()).unwrap();

    #[cfg(feature = "cmake")]
    {
        let mut cxx_cfg = cc::Build::new();
        cxx_cfg
            .flag_if_supported("-Wno-unused-parameter")
            .flag_if_supported("-Wno-attributes")
            .cpp(true)
            .static_crt(true)
            .std("c++17");
        let mut cmake_cfg = cmake::Config::new(_crate_manifest_dir);

        if std::env::var("CARGO_CFG_TARGET_OS").unwrap().as_str() == "windows" {
            cmake_cfg
                .static_crt(true)
                .define("CMAKE_CXX_FLAGS_DEBUG", "/Zi /Ob0 /Od /RTC1")
                .define("CMAKE_CXX_FLAGS_RELEASE", "/O2 /Ob2 /DNDEBUG")
                .define("CMAKE_CXX_FLAGS_MINSIZEREL", "/O1 /Ob1 /DNDEBUG")
                .define("CMAKE_CXX_FLAGS_RELWITHDEBINFO", "/Zi /O2 /Ob1 /DNDEBUG");
        }
        let dst = cmake_cfg.init_cxx_cfg(cxx_cfg).build();
        println!(
            "cargo:rustc-link-search=native={}",
            dst.join("lib").display()
        );
        println!("cargo:rustc-link-lib=static=webrogue_gfxstream");

        #[cfg(target_os = "linux")]
        println!("cargo:rustc-link-lib=dylib=stdc++");
        #[cfg(target_os = "macos")]
        println!("cargo:rustc-link-lib=dylib=c++");
    }
    #[cfg(feature = "cc")]
    {
        let external_dir = _crate_manifest_dir
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("external");
        let gfx_src_dir = external_dir.join("gfxstream");
        let mut build = cc::Build::new();
        build
            .flag_if_supported("-Wno-unused-parameter")
            .flag_if_supported("-Wno-attributes")
            .cpp(true)
            .static_crt(true)
            .std("c++17");

        let sources = vec![
            "$WEBROGUE/webrogue_gfxstream.cpp",
            // host/vulkan
            "external/gfxstream/host/vulkan/VkDecoder.cpp",
            "external/gfxstream/host/vulkan/VulkanStream.cpp",
            "external/gfxstream/host/vulkan/VulkanHandleMapping.cpp",
            "external/gfxstream/host/vulkan/VulkanBoxedHandles.cpp",
            "external/gfxstream/host/vulkan/VkDecoderGlobalState.cpp",
            "external/gfxstream/host/vulkan/RenderThreadInfoVk.cpp",
            "external/gfxstream/host/vulkan/DebugUtilsHelper.cpp",
            "external/gfxstream/host/vulkan/DeviceOpTracker.cpp",
            "external/gfxstream/host/vulkan/VkCommonOperations.cpp",
            "external/gfxstream/host/vulkan/DeviceLostHelper.cpp",
            "external/gfxstream/host/vulkan/VkEmulatedPhysicalDeviceQueue.cpp",
            "external/gfxstream/host/vulkan/VkEmulatedPhysicalDeviceMemory.cpp",
            "external/gfxstream/host/vulkan/VkDecoderSnapshot.cpp",
            "external/gfxstream/host/vulkan/VkReconstruction.cpp",
            "external/gfxstream/host/vulkan/SwapChainStateVk.cpp",
            "external/gfxstream/host/vulkan/DependencyGraph.cpp",
            "external/gfxstream/host/vulkan/VkUtils.cpp",
            // host/vulkan/cereal/common
            "external/gfxstream/host/vulkan/cereal/common/goldfish_vk_dispatch.cpp",
            "external/gfxstream/host/vulkan/cereal/common/goldfish_vk_transform.cpp",
            "external/gfxstream/host/vulkan/cereal/common/goldfish_vk_extension_structs.cpp",
            "external/gfxstream/host/vulkan/cereal/common/goldfish_vk_reserved_marshaling.cpp",
            "external/gfxstream/host/vulkan/cereal/common/goldfish_vk_deepcopy.cpp",
            "external/gfxstream/host/vulkan/cereal/common/goldfish_vk_marshaling.cpp",
            // host/vulkan/emulated_textures
            "external/gfxstream/host/vulkan/emulated_textures/GpuDecompressionPipeline.cpp",
            "external/gfxstream/host/vulkan/emulated_textures/AstcTexture.cpp",
            "external/gfxstream/host/vulkan/emulated_textures/CompressedImageInfo.cpp",
            // host/compressed_textures
            "external/gfxstream/host/compressed_textures/AstcCpuDecompressorNoOp.cpp",
            // host/features
            "external/gfxstream/host/features/Features.cpp",
            // host/backend
            "external/gfxstream/host/backend/external_object_manager.cpp",
            "external/gfxstream/host/backend/vm_operations.cpp",
            "external/gfxstream/host/backend/graphics_driver_lock.cpp",
            // host/health
            "external/gfxstream/host/health/HealthMonitor.cpp",
            // host/metrics
            "external/gfxstream/host/metrics/Metrics.cpp",
            // common/base
            "external/gfxstream/common/base/UdmabufCreator_stub.cpp",
            "external/gfxstream/common/base/Thread_pthread.cpp", // TODO win32
            "external/gfxstream/common/base/System.cpp",
            // common/logging
            "external/gfxstream/common/logging/logging.cpp",
        ];

        for source in sources.iter() {
            let mut parts = source.split('/');
            let mut path = match parts.next().unwrap() {
                "$WEBROGUE" => &_crate_manifest_dir,
                "external" => match parts.next().unwrap() {
                    "gfxstream" => &gfx_src_dir,
                    _ => unimplemented!(),
                },
                _ => unimplemented!(),
            }
            .clone();
            for part in parts {
                path = path.join(part);
            }
            build.file(&path);
            println!("cargo:rerun-if-changed={}", path.display());
        }

        let mut include = |rel_path: &str| {
            let mut path = gfx_src_dir.clone();
            for part in rel_path.split('/') {
                path.push(part);
            }
            build.include(path);
        };
        include("host");
        include("host/vulkan");
        include("host/vulkan/cereal");
        include("host/vulkan/cereal/common");
        include("host/features/include");
        include("host/gl/gl-host-common/include");
        include("host/features/include/gfxstream/host");
        include("host/tracing/include");
        include("host/backend/include");
        include("common/vulkan/include");
        include("common/utils/include");
        include("include");
        include("utils/include");
        include("third-party/renderdoc/include");
        include("third-party/glm/include");
        include("host/compressed_textures/include");
        include("host/health/include");
        include("common/base/include");
        include("host/metrics/include");
        include("common/logging/include");
        include("host/decoder_common/include");
        include("third_party/vulkan/include");
        include("host/include");
        include("host/iostream/include");
        include("third_party/glm/include");
        include("third_party/glm/include");
        include("host/renderdoc/include");
        include("third_party/renderdoc/include");
        include("host/library/include");
        include("host/snapshot/include");
        include("third_party/astc-encoder/Source");

        build
            .define("VK_GFXSTREAM_STRUCTURE_TYPE_EXT", None)
            .compile("webrogue_gfxstream");
    }
}
