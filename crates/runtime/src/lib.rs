use std::sync::{Arc};
// pub use webrogue_wrapp as wrapp;

mod stdout;

#[derive(Debug)]
struct AdditionalImportsBuilder {
    gfx: Arc<webrogue_gfx::GFXSystem>,
}

unsafe impl Sync for AdditionalImportsBuilder {}
unsafe impl Send for AdditionalImportsBuilder {}

impl wasmer_wasix::AdditionalImportsBuilder for AdditionalImportsBuilder {
    fn initialize(
        &self,
        store: &mut dyn wasmer::AsStoreMut,
    ) -> (wasmer::Imports, wasmer_wasix::AdditionalImportsInitializer) {
        let mut import_object = wasmer::Imports::new();
        let gfx_callback = webrogue_gfx::GFXInterface::new(self.gfx.clone())
            .add_to_imports(store, &mut import_object);

        (
            import_object,
            Box::new(move |_instance, memory, _store| {
                gfx_callback(memory.clone())?;

                Ok(())
            }),
        )
    }
}

async fn run_task(
    container: webc::Container,
    stdout: Option<Box<dyn wasmer_wasix::VirtualFile + Send + Sync + 'static>>,
    stderr: Option<Box<dyn wasmer_wasix::VirtualFile + Send + Sync + 'static>>,
    gfx: Arc<webrogue_gfx::GFXSystem>,
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

    #[cfg(feature = "aot")]
    let module = unsafe { wasmer::Module::deserialize(&store, webrogue_aot_data::aot_data())? };
    #[cfg(not(feature = "aot"))]
    let module = {
        let bytecode = container
            .get_atom(&entrypoint_name)
            .ok_or(anyhow::anyhow!("webc entrypoint is not found in container"))?;
        wasmer::Module::new(&store, &bytecode)?
    };

    let mut wasix_runtime = wasmer_wasix::runtime::PluggableRuntime::new(Arc::new(
        wasmer_wasix::runtime::task_manager::tokio::TokioTaskManager::default(),
    ));
    wasix_runtime
        .set_package_loader(wasmer_wasix::runtime::package_loader::BuiltinPackageLoader::new());

    let wasix_runtime_arc = Arc::new(wasix_runtime);

    let mut wasi_env_builder = wasmer_wasix::WasiEnv::builder(entrypoint_name)
        .runtime(wasix_runtime_arc.clone())
        .stdout(stdout.unwrap_or(Box::new(stdout::Stdout::new())))
        .stderr(stderr.unwrap_or(Box::new(stdout::Stdout::new())))
        .import_builder(Arc::new(AdditionalImportsBuilder {
            gfx: gfx,
        }));
    wasi_env_builder.capabilities_mut().threading.enable_blocking_sleep = true;

    // let root_fs = virtual_fs::RootFileSystemBuilder::new()
    //     .with_tty(Box::new(virtual_fs::DeviceFile::new(0)))
    //     .build();

    // let fs_backing: Arc<dyn virtual_fs::FileSystem + Send + Sync> = Arc::new(
    //     virtual_fs::PassthruFileSystem::new(wasmer_wasix::default_fs_backing()),
    // );
    // root_fs.mount(
    //     "/".into(),
    //     &fs_backing,
    //     "/".into(),
    // )?;

    

    wasi_env_builder = wasi_env_builder
        // .sandbox_fs(root_fs)
        .preopen_dir("/")?
        // .map_dir(".", "/")?
        ;

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
    stderr: Option<Box<dyn wasmer_wasix::VirtualFile + Send + Sync + 'static>>,
) -> anyhow::Result<()> {
    let gfx = Arc::new(webrogue_gfx::GFXSystem::new());

    let tokio_runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
    let tokio_guard = tokio_runtime.enter();

    tokio_runtime.block_on(run_task(
        container,
        stdout,
        stderr,
        gfx.clone(),
    ))?;

    drop(tokio_guard);
    drop(gfx);

    return Ok(());
}
