#[cfg(target_os = "windows")]
#[no_mangle]
extern "C" fn webrogue_aot_windows() {
    use std::io::{Read, Seek};
    use webrogue_wasmtime::IVFSBuilder as _;

    let mut current_file = std::fs::File::open(std::env::current_exe().unwrap()).unwrap();
    let file_size = current_file.seek(std::io::SeekFrom::End(0)).unwrap();
    let mut wrapp_size_bytes = [0u8; 8];
    current_file.seek(std::io::SeekFrom::End(-8)).unwrap();
    current_file.read_exact(&mut wrapp_size_bytes).unwrap();
    let wrapp_size = u64::from_le_bytes(wrapp_size_bytes);
    let mut builder = webrogue_wasmtime::WrappVFSBuilder::from_file_part(
        current_file,
        file_size - wrapp_size - 8,
        wrapp_size,
    )
    .unwrap();

    let persistent_path = dirs::data_dir()
        .expect("dirs::data_dir returned None")
        .join(builder.config().unwrap().id.clone().replace('.', "-"))
        .join("persistent");

    webrogue_wasmtime::run_aot_builder(
        webrogue_gfx_winit::SimpleWinitBuilder::default(),
        builder,
        &persistent_path,
    )
    .unwrap();
}

#[cfg(target_os = "linux")]
#[no_mangle]
extern "C" fn webrogue_aot_linux() {
    use std::{io::Seek, os::unix::fs::FileExt};
    use webrogue_wasmtime::IVFSBuilder as _;

    let mut current_file = std::fs::File::open(std::env::current_exe().unwrap()).unwrap();
    let file_size = current_file.seek(std::io::SeekFrom::End(0)).unwrap();
    let mut wrapp_size_bytes = [0u8; 8];
    current_file
        .read_exact_at(&mut wrapp_size_bytes, file_size - 8)
        .unwrap();
    let wrapp_size = u64::from_le_bytes(wrapp_size_bytes);

    let mut builder = webrogue_wasmtime::WrappVFSBuilder::from_file_part(
        current_file,
        file_size - wrapp_size - 8,
        wrapp_size,
    )
    .unwrap();
    let persistent_path = dirs::data_dir()
        .expect("dirs::data_dir returned None")
        .join(builder.config().unwrap().id.clone().replace('.', "-"))
        .join("persistent");

    webrogue_wasmtime::run_aot_builder(
        webrogue_gfx_winit::SimpleWinitBuilder::default(),
        builder,
        &persistent_path,
    )
    .unwrap();
}
