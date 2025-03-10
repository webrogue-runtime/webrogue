use std::sync::Arc;

pub struct GFXInterface {
    gfx: Arc<crate::system::GFXSystem>,
    window: Option<crate::window::Window>,
    gfxstream_thread: Option<webrogue_gfxstream::Thread>,
}

impl GFXInterface {
    pub fn new(gfx: Arc<crate::system::GFXSystem>) -> Self {
        Self {
            gfx,
            window: None,
            gfxstream_thread: None,
        }
    }

    pub fn add_to_imports(
        self,
        store: &mut dyn wasmer::AsStoreMut,
        imports: &mut wasmer::Imports,
    ) -> impl Fn(wasmer::Memory) -> Result<(), anyhow::Error> {
        let lazy = std::rc::Rc::new(crate::env_wrapper::OnceCell::new());
        let env = crate::env_wrapper::EnvWrapper {
            data: self,
            lazy: std::rc::Rc::clone(&lazy),
        };
        let mut store = store.as_store_mut();
        let env = wasmer::FunctionEnv::new(&mut store, env);
        let mut exports = wasmer::Exports::new();
        crate::bindings::add_bindings(&mut exports, store, env);
        imports.register_namespace("webrogue-gfx", exports);
        move |memory: wasmer::Memory| {
            lazy.set(crate::env_wrapper::LazyInitialized { memory: memory })
                .map_err(|_e| anyhow::anyhow!("Couldn't set lazy initialized data"))?;
            Ok(())
        }
    }
}

impl GFXInterface {
    pub fn make_window(&mut self) -> () {
        self.window = Some(self.gfx.make_window());
    }

    pub fn present(&mut self) -> () {
        self.window.as_mut().inspect(|window| {
            window.present();
        });
    }

    pub fn get_window_width(&mut self) -> u32 {
        self.window
            .as_ref()
            .and_then(|window| Some(window.get_size().0))
            .unwrap_or_default()
    }

    pub fn get_window_height(&mut self) -> u32 {
        self.window
            .as_ref()
            .and_then(|window| Some(window.get_size().1))
            .unwrap_or_default()
    }

    pub fn get_gl_width(&mut self) -> u32 {
        self.window
            .as_ref()
            .and_then(|window| Some(window.get_gl_size().0))
            .unwrap_or_default()
    }

    pub fn get_gl_height(&mut self) -> u32 {
        self.window
            .as_ref()
            .and_then(|window| Some(window.get_gl_size().1))
            .unwrap_or_default()
    }

    pub fn gl_init(&mut self) {
        if let Some(window) = &self.window {
            let ret = window.gl_init();
            self.gfxstream_thread = Some(webrogue_gfxstream::Thread::new(ret.0, ret.1))
        }
    }

    pub fn gl_commit_buffer(&mut self, buf: &[u8]) {
        if let Some(gfxstream_thread) = &self.gfxstream_thread {
            gfxstream_thread.commit(buf);
        }
    }

    pub fn gl_ret_buffer_read(&mut self, buf: &mut [u8]) {
        if let Some(gfxstream_thread) = &self.gfxstream_thread {
            gfxstream_thread.read(buf)
        }
    }

    pub fn poll(&self) -> u32 {
        self.gfx.poll()
    }

    pub fn poll_read(&self) -> Option<&'static [u8]> {
        self.gfx.poll_read()
    }
}
