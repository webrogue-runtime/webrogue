@objc
class AngleHelperWindow: NSObject {
    let sdl_window: OpaquePointer
    let sdl_metal_view: SDL_MetalView
    let metal_layer: CAMetalLayer
    private var display: EGLDisplay!
    private var surface: EGLSurface!
    private var context: EGLContext!

    override init() {
        self.sdl_window = SDL_CreateWindow(
            "webrogue",
            WEBROGUE_SDL_WINDOWPOS_UNDEFINED, WEBROGUE_SDL_WINDOWPOS_UNDEFINED,
            800, 450,
            SDL_WINDOW_METAL.rawValue | SDL_WINDOW_RESIZABLE.rawValue | SDL_WINDOW_ALLOW_HIGHDPI.rawValue
        )!
        self.sdl_metal_view = SDL_Metal_CreateView(self.sdl_window)
        self.metal_layer = wr_SDL_Metal_GetLayer(self.sdl_metal_view)
        guard let display = eglGetPlatformDisplay(EGLenum(EGL_PLATFORM_ANGLE_ANGLE), nil, nil) else {
            print("eglGetPlatformDisplay() returned error \(eglGetError())")
            return
        }

        guard eglInitialize(display, nil, nil) != 0 else {
            print("eglInitialize() returned error \(eglGetError())")
            return
        }

        var configAttribs: [EGLint] = [
            EGL_BLUE_SIZE, 8,
            EGL_GREEN_SIZE, 8,
            EGL_RED_SIZE, 8,
            EGL_DEPTH_SIZE, 24,
            EGL_NONE,
        ]

        let configs = UnsafeMutablePointer<EGLConfig?>.allocate(capacity: 1)
        defer { configs.deallocate() }

        var numConfigs: EGLint = 0
        guard eglChooseConfig(display, &configAttribs, configs, 1, &numConfigs) != 0 else {
            print("eglChooseConfig() returned error \(eglGetError())")
            return
        }

        guard let config = configs.pointee else {
            print("Empty config returned in eglChooseConfig()")
            return
        }

        var contextAttribs: [EGLint] = [
            EGL_CONTEXT_MAJOR_VERSION, 3,
            EGL_CONTEXT_MINOR_VERSION, 0,
            EGL_NONE,
        ]

        guard let context = eglCreateContext(display, config, nil, &contextAttribs) else {
            print("eglCreateContext() returned error \(eglGetError())")
            return
        }

        guard let surface = eglCreateWindowSurface(display, config, unsafeBitCast(self.metal_layer, to: EGLNativeWindowType.self), nil) else {
            print("eglCreateWindowSurface() returned error \(eglGetError())")
            return
        }

        self.surface = surface
        self.display = display
        self.context = context
    }

    @objc public func makeCurrent() {
        eglMakeCurrent(display, surface, surface, context)
    }

    @objc public func present() {
        eglSwapBuffers(self.display, self.surface)
    }

    @objc public func viewportWidth() -> Int {
        Int(self.metal_layer.drawableSize.width)
    }

    @objc public func viewportHeight() -> Int {
        Int(self.metal_layer.drawableSize.height)
    }

    @objc public func viewWidth() -> Int {
        Int(self.metal_layer.frame.width)
    }

    @objc public func viewHeight() -> Int {
        Int(self.metal_layer.frame.width)
    }

    @objc public func initGL() {
        SDL_GL_CreateContext(sdl_window);
    }

    deinit {
        DispatchQueue.main.sync {
            SDL_DestroyWindow(sdl_window);
        }
    }
}
