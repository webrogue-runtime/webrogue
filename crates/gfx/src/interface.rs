use std::sync::Arc;

pub struct GFXInterface {
    gfx: Arc<crate::system::GFXSystem>,
    window: Option<crate::window::Window>,
}

impl GFXInterface {
    pub fn new(gfx: Arc<crate::system::GFXSystem>) -> Self {
        #[cfg(feature = "fallback")]
        webrogue_gfx_fallback::dummy();
        Self { gfx, window: None }
    }

    pub fn add_to_imports(
        self,
        store: &mut dyn wasmer::AsStoreMut,
        imports: &mut wasmer::Imports,
    ) -> impl Fn() -> Result<(), anyhow::Error> {
        let env = crate::env_wrapper::EnvWrapper { data: self };
        let mut store = store.as_store_mut();
        let env = wasmer::FunctionEnv::new(&mut store, env);
        let mut exports = wasmer::Exports::new();
        crate::bindings::add_bindings(&mut exports, store, env);
        imports.register_namespace("webrogue-gfx", exports);
        move || Ok(())
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
}
