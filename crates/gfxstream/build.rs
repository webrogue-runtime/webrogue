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
        let aemu_src_dir = external_dir.join("aemu");
        let mut build = cc::Build::new();
        build
            .flag_if_supported("-Wno-unused-parameter")
            .flag_if_supported("-Wno-attributes")
            .cpp(true)
            .static_crt(true)
            .std("c++17");

        let sources = vec![
            "$WEBROGUE/webrogue_gfxstream.cpp",
            // host
            "external/gfxstream/host/ExternalObjectManager.cpp",
            "external/gfxstream/host/GraphicsDriverLock.cpp",
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
            "external/gfxstream/host/vulkan/vk_util.cpp",
            "external/gfxstream/host/vulkan/DeviceLostHelper.cpp",
            "external/gfxstream/host/vulkan/VkEmulatedPhysicalDeviceQueue.cpp",
            "external/gfxstream/host/vulkan/VkEmulatedPhysicalDeviceMemory.cpp",
            "external/gfxstream/host/vulkan/VkDecoderSnapshot.cpp",
            "external/gfxstream/host/vulkan/VkReconstruction.cpp",
            "external/gfxstream/host/vulkan/SwapChainStateVk.cpp",
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
            // host/vulkan/compressedTextureFormats
            "external/gfxstream/host/compressedTextureFormats/AstcCpuDecompressorNoOp.cpp", // TODO impl
            // host/features
            "external/gfxstream/host/features/Features.cpp",
            // aemu
            "external/aemu/base/System.cpp",
            "external/aemu/host-common/GfxstreamFatalError.cpp",
            "external/aemu/host-common/vm_operations.cpp",
            "external/aemu/base/Tracing.cpp",
            "external/aemu/base/Stream.cpp",
            "external/aemu/base/Metrics.cpp",
            "external/aemu/base/HealthMonitor.cpp",
        ];

        for source in sources.iter() {
            let mut parts = source.split('/');
            let mut path = match parts.next().unwrap() {
                "$WEBROGUE" => &_crate_manifest_dir,
                "external" => match parts.next().unwrap() {
                    "gfxstream" => &gfx_src_dir,
                    "aemu" => &aemu_src_dir,
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

        // .file(aemu_src_dir.join("host-common").join("vm_operations.cpp"))
        // .file(aemu_src_dir.join("host-common").join("crash_reporter.cpp"))
        build
            .include(gfx_src_dir.join("host"))
            .include(gfx_src_dir.join("host").join("vulkan"))
            .include(gfx_src_dir.join("host").join("vulkan").join("cereal"))
            .include(
                gfx_src_dir
                    .join("host")
                    .join("vulkan")
                    .join("cereal")
                    .join("common"),
            )
            .include(gfx_src_dir.join("host").join("features").join("include"))
            .include(
                gfx_src_dir
                    .join("host")
                    .join("gl")
                    .join("gl-host-common")
                    .join("include"),
            )
            .include(
                gfx_src_dir
                    .join("host")
                    .join("features")
                    .join("include")
                    .join("gfxstream")
                    .join("host"),
            )
            .include(gfx_src_dir.join("host").join("tracing").join("include"))
            .include(gfx_src_dir.join("host").join("backend").join("include"))
            .include(gfx_src_dir.join("common").join("vulkan").join("include"))
            .include(gfx_src_dir.join("common").join("utils").join("include"))
            .include(gfx_src_dir.join("include"))
            .include(gfx_src_dir.join("utils").join("include"))
            .include(
                gfx_src_dir
                    .join("third-party")
                    .join("renderdoc")
                    .join("include"),
            )
            .include(gfx_src_dir.join("third-party").join("glm").join("include"))
            // .include(gfx_src_dir.join("third-party").join("astc-encoder").join("Source"))
            .include(aemu_src_dir.join("snapshot").join("include"))
            .include(aemu_src_dir.join("base").join("include"))
            .include(aemu_src_dir.join("host-common").join("include"))
            .define("VK_GFXSTREAM_STRUCTURE_TYPE_EXT", None)
            .compile("webrogue_gfxstream");
    }
}
