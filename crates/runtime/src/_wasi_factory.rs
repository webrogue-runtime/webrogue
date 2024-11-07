pub trait WasiFactory {
    fn make(&self) -> wasi_common::WasiCtx;
    fn add_actual_dir(
        &self,
        wasi: &mut wasi_common::WasiCtx,
        actual_path: std::path::PathBuf,
        guest_path: &str,
    );
}
