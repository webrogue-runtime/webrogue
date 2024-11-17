use std::sync::{Arc, RwLock};
// pub use webrogue_wrapp as wrapp;

#[cfg(feature = "aot")]
extern "C" {
    static WASMER_METADATA_WR_AOT: u8;
    static WASMER_METADATA_WR_AOT_SIZE: usize;
}

#[derive(Debug)]
struct AdditionalImportsBuilder {
    #[cfg(feature = "gfx")]
    gfx: Arc<webrogue_gfx::GFXSystem>,
    #[cfg(feature = "gl")]
    gl: Arc<RwLock<webrogue_gl::GL>>,
}

unsafe impl Sync for AdditionalImportsBuilder {}
unsafe impl Send for AdditionalImportsBuilder {}

impl wasmer_wasix::AdditionalImportsBuilder for AdditionalImportsBuilder {
    fn initialize(
        &self,
        store: &mut dyn wasmer::AsStoreMut,
    ) -> (wasmer::Imports, wasmer_wasix::AdditionalImportsInitializer) {
        let mut import_object = wasmer::Imports::new();
        #[cfg(feature = "gfx")]
        let gfx_callback = webrogue_gfx::GFXInterface::new(self.gfx.clone())
            .add_to_imports(store, &mut import_object);

        #[cfg(feature = "gl")]
        let gl_callback = webrogue_gl::add_to_imports(store, &mut import_object, self.gl.clone());

        (
            import_object,
            Box::new(move |instance, memory, store| {
                #[cfg(feature = "gfx")]
                gfx_callback()?;
                #[cfg(feature = "gl")]
                gl_callback(memory.clone())?;

                Ok(())
            }),
        )
    }
}

async fn run_task(
    container: webc::Container,
    stdout: Option<Box<dyn wasmer_wasix::VirtualFile + Send + Sync + 'static>>,
    #[cfg(feature = "gfx")] gfx: Arc<webrogue_gfx::GFXSystem>,
    #[cfg(feature = "gl")] gl: Arc<RwLock<webrogue_gl::GL>>,
) -> anyhow::Result<()> {
    let store = wasmer::Store::default();

    // let mut wasm_file = wrapp_handle.open_file("main.wasm").unwrap();
    let entrypoint_name = container
        .manifest()
        .entrypoint
        .clone()
        .ok_or(anyhow::anyhow!("webc entrypoint is not specified"))?;
    let _entrypoint_atom = container
        .manifest()
        .atoms
        .get(&entrypoint_name)
        .ok_or(anyhow::anyhow!("webc entrypoint is not found in commands"))?;
    let bytecode = container
        .get_atom(&entrypoint_name)
        .ok_or(anyhow::anyhow!("webc entrypoint is not found in container"))?;
    // let mut bytecode = Vec::new();
    // wasm_file.read_to_end(&mut bytecode)?;

    #[cfg(feature = "aot")]
    let module = unsafe {
        wasmer::Module::deserialize(
            &store,
            std::slice::from_raw_parts(&WASMER_METADATA_WR_AOT, WASMER_METADATA_WR_AOT_SIZE),
        )?
    };
    #[cfg(not(feature = "aot"))]
    let module = wasmer::Module::new(&store, &bytecode)?;

    let mut wasix_runtime = wasmer_wasix::runtime::PluggableRuntime::new(Arc::new(
        wasmer_wasix::runtime::task_manager::tokio::TokioTaskManager::default(),
    ));
    wasix_runtime
        .set_package_loader(wasmer_wasix::runtime::package_loader::BuiltinPackageLoader::new());

    let wasix_runtime_arc = Arc::new(wasix_runtime);

    let mut wasi_env_builder = wasmer_wasix::WasiEnv::builder(entrypoint_name)
        .runtime(wasix_runtime_arc.clone())
        .stdout(stdout.unwrap_or(Box::new(virtual_fs::host_fs::Stdout::default())))
        .import_builder(Arc::new(AdditionalImportsBuilder {
            #[cfg(feature = "gfx")]
            gfx: gfx,
            #[cfg(feature = "gl")]
            gl: gl,
        }));

    wasi_env_builder.add_webc(
        wasmer_wasix::bin_factory::BinaryPackage::from_webc(&container, wasix_runtime_arc.as_ref())
            .await?,
    );

    wasi_env_builder.run(module)?;

    Ok(())
}

pub fn run(
    container: webc::Container,
    stdout: Option<Box<dyn wasmer_wasix::VirtualFile + Send + Sync + 'static>>,
) -> anyhow::Result<()> {
    #[cfg(feature = "gfx")]
    let gfx = Arc::new(webrogue_gfx::GFXSystem::new());

    #[cfg(feature = "gl")]
    let gl = Arc::new(RwLock::new(webrogue_gl::GL::new(gfx.clone())));

    let tokio_runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
    let tokio_guard = tokio_runtime.enter();

    tokio_runtime.block_on(run_task(container, stdout, gfx.clone(), gl))?;

    drop(tokio_guard);
    #[cfg(feature = "gfx")]
    drop(gfx);

    return Ok(());
}
