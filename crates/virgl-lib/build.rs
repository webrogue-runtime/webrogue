#[cfg(not(feature = "_build"))]
fn main() {}

#[cfg(feature = "_build")]
fn main() {
    use std::collections::HashSet;
    use std::env;

    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    if target_arch == "wasm32" {
        return;
    }

    use std::fs::File;
    use std::io::Read;
    use std::io::Write as _;
    use std::str::FromStr as _;
    let _crate_manifest_dir =
        std::path::PathBuf::from_str(&std::env::var("CARGO_MANIFEST_DIR").unwrap()).unwrap();
    let _os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();

    let external_dir = _crate_manifest_dir
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("external");
    let out_dir = std::path::PathBuf::from_str(&env::var("OUT_DIR").unwrap()).unwrap();
    let virgl_src_dir = external_dir.join("virglrenderer");
    let mut build = cc::Build::new();
    build
        .flag_if_supported("-Wno-unused-parameter")
        .flag_if_supported("-Wno-attributes");

    // Only use static_crt on Windows
    if _os == "windows" {
        build.static_crt(true);
    }

    let mut meson_build_file = "".to_string();
    File::open(external_dir.join("virglrenderer").join("meson.build"))
        .unwrap()
        .read_to_string(&mut meson_build_file)
        .unwrap();

    let mut is_windows = false;
    let mut _is_macos = false;
    let mut is_linux = false;
    let mut _is_android = false;
    let mut is_freebsd = false;
    match _os.as_str() {
        "windows" => {
            is_windows = true;
        }
        "macos" => {
            _is_macos = true;
        }
        "linux" => {
            is_linux = true;
        }
        "android" => {
            _is_android = true;
            is_linux = true;
        }
        "freebsd" => {
            is_freebsd = true;
        }
        _ => unimplemented!(),
    };

    let virglrenderer_version = meson_build_file
        .lines()
        .find(|line| line.contains("version:"))
        .unwrap()
        .split(':')
        .nth(1)
        .unwrap()
        .replace('\'', "")
        .replace(',', "");
    let virglrenderer_version = virglrenderer_version.trim();

    fn def_if(condition: bool) -> Option<&'static str> {
        if condition {
            Some("1")
        } else {
            None
        }
    }

    let mut bindgen_args = Vec::new();

    let config: Vec<(&str, Option<&str>)> = vec![
        ("VERSION", Some(virglrenderer_version)),
        ("_GNU_SOURCE", Some("1")),
        ("VIRGL_RENDERER_UNSTABLE_APIS", Some("1")),
        ("HAVE___BUILTIN_BSWAP32", Some("1")),
        ("HAVE___BUILTIN_BSWAP64", Some("1")),
        ("HAVE___BUILTIN_CLZ", Some("1")),
        ("HAVE___BUILTIN_CLZLL", Some("1")),
        ("HAVE___BUILTIN_EXPECT", Some("1")),
        ("HAVE___BUILTIN_FFS", Some("1")),
        ("HAVE___BUILTIN_FFSLL", Some("1")),
        ("HAVE___BUILTIN_POPCOUNT", Some("1")),
        ("HAVE___BUILTIN_POPCOUNTLL", Some("1")),
        ("HAVE___BUILTIN_TYPES_COMPATIBLE_P", Some("1")),
        ("HAVE___BUILTIN_UNREACHABLE", Some("1")),
        ("HAVE_FUNC_ATTRIBUTE_CONST", Some("1")),
        ("HAVE_FUNC_ATTRIBUTE_FLATTEN", Some("1")),
        ("HAVE_FUNC_ATTRIBUTE_FORMAT", Some("1")),
        ("HAVE_FUNC_ATTRIBUTE_MALLOC", Some("1")),
        ("HAVE_FUNC_ATTRIBUTE_NORETURN", Some("1")),
        ("HAVE_FUNC_ATTRIBUTE_PACKED", Some("1")),
        ("HAVE_FUNC_ATTRIBUTE_PURE", Some("1")),
        ("HAVE_FUNC_ATTRIBUTE_RETURNS_NONNULL", Some("1")),
        ("HAVE_FUNC_ATTRIBUTE_UNUSED", Some("1")),
        ("HAVE_FUNC_ATTRIBUTE_WARN_UNUSED_RESULT", Some("1")),
        ("HAVE_FUNC_ATTRIBUTE_WEAK", Some("1")),
        ("HAVE_MEMFD_CREATE", def_if(is_linux)),
        ("HAVE_STRTOK_R", def_if(!is_windows)),
        ("HAVE_TIMESPEC_GET", Some("1")),
        ("HAVE_SYS_UIO_H", Some("1")),
        ("HAVE_PTHREAD", def_if(!is_windows)),
        ("HAVE_PTHREAD_SETAFFINITY", def_if(!is_windows)),
        ("HAVE_PTHREAD_NP_H", def_if(is_freebsd)),
        ("HAVE_EPOXY_EGL_H", None),
        ("HAVE_EPOXY_GLX_H", None),
        ("CHECK_GL_ERRORS", Some("1")),
        ("ENABLE_GBM_ALLOCATION", None),
        ("ENABLE_VENUS", Some("1")),
        ("ENABLE_VULKAN_DLOAD", None),
        ("ENABLE_VULKAN_PRELOAD", None),
        ("ENABLE_GBM", None),
        ("ENABLE_DRM", None),
        ("ENABLE_DRM_MSM", None),
        ("ENABLE_DRM_AMDGPU", None),
        ("ENABLE_DRM_ASAHI", None),
        ("ENABLE_DRM_PANFROST", None),
        ("ENABLE_DRM_I915", None),
        ("ENABLE_LIBDRM", None),
        ("ENABLE_RENDER_SERVER", Some("1")),
        ("ENABLE_SAME_PROCESS_RENDER_SERVER", Some("1")),
        ("ENABLE_RENDER_SERVER_WORKER_PROCESS", None),
        ("ENABLE_RENDER_SERVER_WORKER_THREAD", Some("1")),
        ("ENABLE_RENDER_SERVER_WORKER_MINIJAIL", None),
        ("RENDER_SERVER_EXEC_PATH", Some("\"No path to that\"")),
        ("HAVE_EVENTFD_H", def_if(is_linux)),
        ("HAVE_DMABUF_H", None),
        ("HAVE_LINUX_UDMABUF_H", None),
        ("HAVE_DLFCN_H", None),
        ("ENABLE_VIDEO", None),
        ("ENABLE_TRACING", None),
        ("ENABLE_TESTS", None),
        ("UTIL_ARCH_LITTLE_ENDIAN", Some("1")),
        ("UTIL_ARCH_BIG_ENDIAN", Some("0")),
        ("PIPE_ARCH_X86", def_if(target_arch == "x86")),
        ("PIPE_ARCH_X86_64", def_if(target_arch == "x86_64")),
        ("PIPE_ARCH_PPC", def_if(target_arch == "powerpc")),
        ("PIPE_ARCH_PPC_64", def_if(target_arch == "powerpc64")),
        ("PIPE_ARCH_S390", def_if(target_arch == "s390")),
        ("PIPE_ARCH_ARM", def_if(target_arch == "arm")),
        ("PIPE_ARCH_AARCH64", def_if(target_arch == "aarch64")),
    ];

    let mut config_file_in = "".to_string();
    File::open(external_dir.join("virglrenderer").join("config.h.meson"))
        .unwrap()
        .read_to_string(&mut config_file_in)
        .unwrap();

    let needed_config_keys = config_file_in
        .split("#mesondefine")
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect::<HashSet<_>>();

    let difference = config
        .iter()
        .map(|c| c.0)
        .collect::<HashSet<_>>()
        .difference(&needed_config_keys)
        .cloned()
        .collect::<Vec<_>>();
    if !difference.is_empty() {
        panic!("The following values are unneeded: {:?}", difference)
    }
    let mut config_file_out = "".to_string();
    for (k, v) in config {
        let Some(v) = v else {
            continue;
        };
        config_file_out.push_str(&format!("#define {} {}\n", k, v));
    }
    let config_h_path = out_dir.join("config.h");
    {
        let mut config_file_out_old = String::new();
        let _ = File::open(&config_h_path)
            .and_then(|mut file| file.read_to_string(&mut config_file_out_old));
        if config_file_out_old != config_file_out {
            File::create(&config_h_path)
                .unwrap()
                .write_all(config_file_out.as_bytes())
                .unwrap();
        }
    }

    build.define("HAVE_CONFIG_H", "1");
    if is_windows {
        // clang-cl / MSVC's cl ignore clang's `-imacros`; use the MSVC
        // force-include form instead so every translation unit sees config.h.
        build.flag(format!("/FI{}", config_h_path.display()));
    } else {
        build.flag(format!("-imacros{}", config_h_path.display()));
    }
    build.include(&out_dir);

    if is_windows {
        // Windows POSIX compatibility layer (see crates/virgl-lib/compat).
        let compat_dir = _crate_manifest_dir.join("compat");
        build.file(&compat_dir.join("win32_compat.c"));
        build.include(&compat_dir);
        // Force-include the compat prelude (clock_gettime etc.) in every TU.
        build.flag(format!("/FI{}", compat_dir.join("prelude.h").display()));
        // cc only tracks .file() inputs, not headers this crate adds to the
        // include path; make sure compat header edits trigger a rebuild.
        fn track_compat(dir: &std::path::Path) {
            for entry in std::fs::read_dir(dir).unwrap() {
                let path = entry.unwrap().path();
                if path.is_dir() {
                    track_compat(&path);
                } else {
                    println!("cargo:rerun-if-changed={}", path.display());
                }
            }
        }
        track_compat(&compat_dir);
    }

    bindgen_args.push(format!("-DHAVE_CONFIG_H=1"));
    bindgen_args.push(format!("-imacros{}", config_h_path.display()));
    bindgen_args.push(format!("-I{}", out_dir.display()));
    if is_windows {
        bindgen_args.push(format!(
            "-I{}",
            _crate_manifest_dir.join("compat").display()
        ));
    }

    {
        use std::io::Write as _;

        let mut config = "".to_string();
        File::open(
            external_dir
                .join("virglrenderer")
                .join("src")
                .join("virgl-version.h.meson"),
        )
        .unwrap()
        .read_to_string(&mut config)
        .unwrap();
        config = config
            .replace(
                "(@VIRGL_MAJOR_VERSION@)",
                virglrenderer_version.split(".").nth(0).unwrap(),
            )
            .replace(
                "(@VIRGL_MINOR_VERSION@)",
                virglrenderer_version.split(".").nth(1).unwrap(),
            )
            .replace(
                "(@VIRGL_MICRO_VERSION@)",
                virglrenderer_version.split(".").nth(2).unwrap(),
            );

        {
            let config_h_path = out_dir.join("virgl-version.h");
            let mut config_file_out_old = String::new();
            let _ = File::open(&config_h_path)
                .and_then(|mut file| file.read_to_string(&mut config_file_out_old));
            if config_file_out_old != config {
                File::create(&config_h_path)
                    .unwrap()
                    .write_all(config.as_bytes())
                    .unwrap();
            }
        }
    }

    if std::env::var("DEBUG").unwrap() == "true" {
        build.define("DEBUG", "1");
    }

    let mut sources = Vec::new();
    #[cfg(feature = "impl")]
    {
        sources.append(&mut vec![
            "$CRATE/webrogue_virgl_impl.c",
            "external/virglrenderer/src/virglrenderer.c",
            "external/virglrenderer/src/virgl_fence.c",
            "external/virglrenderer/src/virgl_util.c",
            "external/virglrenderer/src/virgl_context.c",
            "external/virglrenderer/src/virgl_resource.c",
            "external/virglrenderer/src/venus/vkr_allocator.c",
            "external/virglrenderer/src/mesa/util/hash_table.c",
            "external/virglrenderer/src/mesa/util/ralloc.c",
            "external/virglrenderer/src/mesa/util/os_file.c",
            "external/virglrenderer/src/mesa/util/u_debug.c",
            "external/virglrenderer/src/mesa/util/os_misc.c",
            "external/virglrenderer/src/mesa/util/anon_file.c",
            "external/virglrenderer/src/gallium/auxiliary/util/u_hash_table.c",
            "external/virglrenderer/src/gallium/auxiliary/util/u_debug_describe.c",
            "external/virglrenderer/src/gallium/auxiliary/util/u_texture.c",
            "external/virglrenderer/src/gallium/auxiliary/util/u_format.c",
            "external/virglrenderer/src/gallium/auxiliary/tgsi/tgsi_info.c",
            "external/virglrenderer/src/gallium/auxiliary/tgsi/tgsi_scan.c",
            "external/virglrenderer/src/gallium/auxiliary/tgsi/tgsi_dump.c",
            "external/virglrenderer/src/gallium/auxiliary/tgsi/tgsi_strings.c",
            "external/virglrenderer/src/gallium/auxiliary/tgsi/tgsi_text.c",
            "external/virglrenderer/src/gallium/auxiliary/tgsi/tgsi_util.c",
            "external/virglrenderer/src/gallium/auxiliary/tgsi/tgsi_parse.c",
            "external/virglrenderer/src/gallium/auxiliary/tgsi/tgsi_sanity.c",
            "external/virglrenderer/src/gallium/auxiliary/tgsi/tgsi_build.c",
            "external/virglrenderer/src/gallium/auxiliary/tgsi/tgsi_iterate.c",
            "external/virglrenderer/src/gallium/auxiliary/cso_cache/cso_hash.c",
            "external/virglrenderer/src/gallium/auxiliary/cso_cache/cso_cache.c",
            "external/virglrenderer/gen/u_format_table.c",
            "external/virglrenderer/src/vrend/vrend_blitter.c",
            "external/virglrenderer/src/vrend/vrend_formats.c",
            "external/virglrenderer/src/vrend/vrend_tweaks.c",
            "external/virglrenderer/src/vrend/vrend_debug.c",
            "external/virglrenderer/src/vrend/vrend_object.c",
            "external/virglrenderer/src/vrend/vrend_decode.c",
            "external/virglrenderer/src/vrend/vrend_shader.c",
            "external/virglrenderer/src/vrend/iov.c",
            "external/virglrenderer/src/vrend/vrend_renderer.c",
            "external/virglrenderer/src/vrend/vrend_winsys.c",
            "external/virglrenderer/src/venus/vkr_renderer.c",
            "external/virglrenderer/src/venus/vkr_buffer.c",
            "external/virglrenderer/src/venus/vkr_common.c",
            "external/virglrenderer/src/venus/vkr_context.c",
            "external/virglrenderer/src/venus/vkr_command_buffer.c",
            "external/virglrenderer/src/venus/vkr_cs.c",
            "external/virglrenderer/src/venus/vkr_instance.c",
            "external/virglrenderer/src/venus/vkr_device.c",
            "external/virglrenderer/src/venus/vkr_transport.c",
            "external/virglrenderer/src/venus/vkr_ring.c",
            "external/virglrenderer/src/venus/vkr_physical_device.c",
            "external/virglrenderer/src/venus/vkr_image.c",
            "external/virglrenderer/src/venus/vkr_library.c",
            "external/virglrenderer/src/venus/vkr_descriptor_heap.c",
            "external/virglrenderer/src/venus/vkr_queue.c",
            "external/virglrenderer/src/venus/vkr_query_pool.c",
            "external/virglrenderer/src/venus/vkr_descriptor_set.c",
            "external/virglrenderer/src/venus/vkr_pipeline.c",
            "external/virglrenderer/src/venus/vkr_acceleration_structure.c",
            "external/virglrenderer/src/venus/vkr_render_pass.c",
            "external/virglrenderer/src/venus/vkr_device_memory.c",
            "external/virglrenderer/src/venus/vkr_host_copy.c",
            "external/virglrenderer/src/venus/vkr_webrogue.c",
            "external/virglrenderer/src/proxy/proxy_renderer.c",
            "external/virglrenderer/src/proxy/proxy_server.c",
            "external/virglrenderer/src/proxy/proxy_client.c",
            "external/virglrenderer/src/proxy/proxy_context.c",
            "external/virglrenderer/src/proxy/proxy_socket.c",
            "external/virglrenderer/src/proxy/proxy_common.c",
            "external/virglrenderer/server/render_server.c",
            "external/virglrenderer/server/render_client.c",
            "external/virglrenderer/server/render_context.c",
            "external/virglrenderer/server/render_socket.c",
            "external/virglrenderer/server/render_worker.c",
            "external/virglrenderer/server/render_state.c",
            "external/virglrenderer/server/render_common.c",
        ]);
    };
    #[cfg(feature = "stub")]
    {
        sources.push("$CRATE/webrogue_virgl_stub.c");
    }

    for source in sources.iter() {
        let mut parts = source.split('/');
        let mut path = match parts.next().unwrap() {
            "$CRATE" => &_crate_manifest_dir,
            "external" => match parts.next().unwrap() {
                "virglrenderer" => &virgl_src_dir,
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

    let includes = [
        ".",
        "gen",
        "src",
        "src/venus",
        "src/drm",
        "src/drm/drm-uapi",
        "src/gallium/include",
        "src/gallium/auxiliary",
        "src/gallium/auxiliary/util",
        "src/mesa",
        "src/mesa/pipe",
        "src/mesa/compat",
        "src/mesa/util",
    ];

    for rel_path in includes {
        let mut path = virgl_src_dir.clone();
        for part in rel_path.split('/') {
            path.push(part);
        }
        if !path.exists() {
            panic!("{} ({}) does not exists", path.display(), rel_path);
        }
        build.include(&path);
        bindgen_args.push(format!("-I{}", path.display()));
    }
    build.include(&_crate_manifest_dir);

    #[cfg(not(target_env = "musl"))]
    bindgen::Builder::default()
        .header(
            _crate_manifest_dir
                .join("webrogue_virgl.h")
                .to_str()
                .unwrap(),
        )
        .clang_args(bindgen_args)
        .raw_line("#![allow(dead_code)]")
        .raw_line("#![allow(nonstandard_style)]")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .unwrap()
        .write_to_file(
            _crate_manifest_dir
                .parent()
                .unwrap()
                .join("virgl")
                .join("src")
                .join("bindings.rs"),
        )
        .unwrap();

    build
        .define("VK_USE_PLATFORM_WEBROGUE", None)
        .compile("webrogue_virgl");
}
