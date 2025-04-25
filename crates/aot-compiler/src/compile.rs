use std::collections::BTreeMap;
use webrogue_wrapp::IVFSHandle as _;

pub fn compile_wrapp_to_object(
    wrapp_file_path: &std::path::PathBuf,
    object_file_path: &std::path::PathBuf,
    target: crate::Target,
    is_pic: bool,
) -> anyhow::Result<()> {
    let mut config = wasmtime::Config::new();
    config.target(target.name())?;
    config.cranelift_opt_level(wasmtime::OptLevel::SpeedAndSize);
    config.cranelift_regalloc_algorithm(wasmtime::RegallocAlgorithm::Backtracking);
    config.wasm_backtrace_details(wasmtime::WasmBacktraceDetails::Disable);
    config.generate_address_map(false);
    config.epoch_interruption(false);
    unsafe {
        if is_pic {
            config.cranelift_flag_enable("is_pic");
        }
    }
    let engine = wasmtime::Engine::new(&config)?;

    let vfs = webrogue_wrapp::WrappVFSBuilder::from_file_path(wrapp_file_path)?.build()?;

    let mut file = vfs
        .open_file("/app/main.wasm")?
        .ok_or(anyhow::anyhow!("/app/main.wasm not found"))?;
    let mut wasm_binary = Vec::new();
    std::io::Read::read_to_end(&mut file, &mut wasm_binary)?;
    drop(file);

    let cwasm = engine.precompile_module(&wasm_binary)?;

    let cwasm_info = crate::cwasm_analyzer::analyze_cwasm(&cwasm)?;

    let mut obj = object::write::Object::new(target.format(), target.arch(), target.endianness());

    obj.add_file_symbol(b"/app/main.wasm".into());
    let mut main_data = Vec::new();
    main_data.extend_from_slice(&(cwasm.len() as u64).to_le_bytes());
    main_data.extend_from_slice(&(cwasm_info.max_alignment).to_le_bytes());
    main_data.extend_from_slice(&vec![0u8].repeat(cwasm_info.max_alignment as usize - 16));
    main_data.extend_from_slice(&cwasm);

    let main_symbol = obj.add_symbol(object::write::Symbol {
        name: b"WEBROGUE_AOT".into(),
        value: 0,
        size: 0,
        kind: object::SymbolKind::Text,
        scope: object::SymbolScope::Linkage,
        weak: false,
        section: object::write::SymbolSection::Undefined,
        flags: object::SymbolFlags::None,
    });
    // Add the main function in its own subsection (equivalent to -ffunction-sections).
    let main_section = obj.add_subsection(object::write::StandardSection::Text, b"main");
    let _main_offset = obj.add_symbol_data(
        main_symbol,
        main_section,
        &main_data,
        cwasm_info.max_alignment,
    );

    let mut writer = std::io::BufWriter::new(std::fs::File::create(&object_file_path)?);
    obj.write_stream(&mut writer)
        .map_err(|err| anyhow::anyhow!(err.to_string()))?;
    std::io::Write::flush(&mut writer)?;

    Ok(())
}
