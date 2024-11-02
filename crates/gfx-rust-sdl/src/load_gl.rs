pub fn load_gl(video_subsystem: &sdl2::VideoSubsystem) {
    #[cfg(target_os = "macos")]
    {
        let result = (|| {
            let path = std::env::current_exe().ok()?;
            let path = path.parent()?;

            if path.join("libGLESv2.dylib").exists() && path.join("libEGL.dylib").exists() {
                std::env::set_var("SDL_VIDEO_EGL_DRIVER", path.join("libEGL.dylib"));
                std::env::set_var("SDL_VIDEO_GL_DRIVER", path.join("libGLESv2.dylib"));
                return Some(());
            } else {
                return None;
            }
        })();

        if result.is_some() {
            return;
        }
    }
    #[cfg(target_os = "windows")]
    {
        let result = (|| {
            let path = std::env::current_exe().ok()?;
            let path = path.parent()?;

            if path.join("libGLESv2.dll").exists() && path.join("libEGL.dll").exists() {
                std::env::set_var("SDL_VIDEO_EGL_DRIVER", path.join("libEGL.dll"));
                std::env::set_var("SDL_OPENGL_LIBRARY", path.join("libGLESv2.dll"));
                std::env::set_var("SDL_VIDEO_GL_DRIVER", path.join("libGLESv2.dll"));
                return Some(());
            } else {
                return None;
            }
        })();

        if result.is_some() {
            return;
        }
    }

    let result = video_subsystem.gl_load_library_default();
    if result.is_ok() {
        return;
    }
}
