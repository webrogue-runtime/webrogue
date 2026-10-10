#[cfg(feature = "cache")]
use std::path::{Path, PathBuf};

use crate::{runtime::run_component, Runtime};

pub struct JitRuntime {
    base: Runtime,
    #[cfg(feature = "cache")]
    jit_cache_config: Option<PathBuf>,
    jit_profile: JitProfile,
    is_panic_allowed: bool,
}

impl JitRuntime {
    pub(crate) fn new(base: Runtime) -> Self {
        Self {
            base,
            #[cfg(feature = "cache")]
            jit_cache_config: None,
            jit_profile: JitProfile::FastExecution,
            is_panic_allowed: false,
        }
    }

    pub fn jit_profile(&mut self, value: JitProfile) -> &mut Self {
        self.jit_profile = value;
        self
    }

    #[cfg(feature = "cache")]
    pub fn jit_cache_config(&mut self, value: &Path) -> &mut Self {
        self.jit_cache_config = Some(value.to_path_buf());
        self
    }

    pub unsafe fn allow_panic(&mut self) -> &mut Self {
        self.is_panic_allowed = true;
        self
    }

    #[cfg(feature = "debug")]
    pub fn debug_connection_factory(
        &mut self,
        value: webrogue_debugger::connection::ConnectionFactory,
    ) -> &mut Self {
        self.base.debug_connection_factory = Some(value);
        self
    }

    pub async fn run(mut self) -> anyhow::Result<()> {
        use std::io::Read as _;

        use wasmtime::Inlining;
        use webrogue_vfs::MAIN_WASM_VFS_PATH;

        self.base
            .wasmtime_config
            .wasm_backtrace_details(wasmtime::WasmBacktraceDetails::Enable)
            .debug_info(false);

        #[cfg(feature = "cache")]
        if let Some(cache_config) = self.jit_cache_config {
            let cache_config = if cache_config.is_absolute() {
                cache_config
            } else {
                std::path::absolute(cache_config)?
            };
            std::fs::create_dir_all(&cache_config)?;
            let mut cache = wasmtime::CacheConfig::new();
            cache.with_directory(&cache_config);
            self.base
                .wasmtime_config
                .cache(Some(wasmtime::Cache::new(cache)?));
            // TODO config.enable_incremental_compilation(cache_store)
        }
        #[cfg(feature = "debug")]
        if self.base.debug_connection_factory.is_some() {
            anyhow::ensure!(matches!(&self.jit_profile, JitProfile::Debug))
        }
        match &self.jit_profile {
            #[cfg(feature = "debug")]
            JitProfile::Debug => {
                self.base
                    .wasmtime_config
                    .cranelift_opt_level(wasmtime::OptLevel::None)
                    .guest_debug(true)
                    .cranelift_regalloc_algorithm(wasmtime::RegallocAlgorithm::SinglePass)
                    .compiler_inlining(Inlining::No)
                    .max_wasm_stack(2 * 1024 * 1024)
                    .async_stack_size(4 * 1024 * 1024);
            }
            JitProfile::FastExecution => {
                self.base
                    .wasmtime_config
                    .cranelift_opt_level(wasmtime::OptLevel::Speed)
                    .cranelift_regalloc_algorithm(wasmtime::RegallocAlgorithm::Backtracking)
                    .compiler_inlining(Inlining::Intrinsics)
                    .signals_based_traps(true);
                webrogue_virgl::shadow_blob::external_signal_handler_installed();
            }
            JitProfile::FastCompilation => {
                self.base
                    .wasmtime_config
                    .cranelift_opt_level(wasmtime::OptLevel::Speed)
                    .cranelift_regalloc_algorithm(wasmtime::RegallocAlgorithm::SinglePass)
                    .compiler_inlining(Inlining::Intrinsics)
                    .signals_based_traps(true);
                webrogue_virgl::shadow_blob::external_signal_handler_installed();
            }
        };

        let enable_epoch_interruption = !self.is_panic_allowed || {
            #[cfg(feature = "debug")]
            {
                matches!(self.jit_profile, JitProfile::Debug)
            }
            #[cfg(not(feature = "debug"))]
            false
        };
        self.base
            .wasmtime_config
            .epoch_interruption(enable_epoch_interruption);
        let engine = wasmtime::Engine::new(&self.base.wasmtime_config)?;
        let mut wasm_binary = Vec::new();
        self.base
            .vfs
            .open(MAIN_WASM_VFS_PATH)
            .map_err(|_| anyhow::anyhow!("Unable to open WASM binary in VFS"))?
            .reader()
            .read_to_end(&mut wasm_binary)?;

        let component = wasmtime::component::Component::from_binary(&engine, &wasm_binary)?;
        run_component(self.base, engine, component).await
    }
}

pub enum JitProfile {
    #[cfg(feature = "debug")]
    Debug,
    FastExecution,
    FastCompilation,
}
