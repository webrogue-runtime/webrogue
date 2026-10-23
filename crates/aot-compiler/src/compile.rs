use std::io::Write as _;

use wasmtime::Cache;
use webrogue_vfs::{MAIN_WASM_VFS_PATH, VFS};

use crate::utils::TemporaryFile;

pub fn process_wasm(
    wrapp_file_path: &std::path::Path,
    tmp: &std::path::Path,
) -> anyhow::Result<Vec<u8>> {
    let tmp_dir = if tmp.is_dir() {
        tmp
    } else {
        tmp.parent().ok_or(anyhow::anyhow!("Path error"))?
    };

    let vfs = VFS::build_from_path(wrapp_file_path)?;

    let component = vfs
        .read_as_vec(MAIN_WASM_VFS_PATH)
        .map_err(|_| anyhow::anyhow!("Unable to read WebAssembly code"))?;
    let component = webrogue_wasm_processing::compose(component)?;

    let tmp_file_1 = TemporaryFile::new(tmp_dir)?;
    let tmp_file_2 = TemporaryFile::new(tmp_dir)?;
    let optimized_component =
        webrogue_wasm_processing::optimize(&component, tmp_file_1.path(), tmp_file_2.path())?;
    let component = optimized_component.unwrap_or_else(|| {
        webrogue_cli_goodies::warning(
            "optimization is skipped because `wasm-opt` feature is disabled at compile time",
        );
        component
    });
    Ok(component)
}

pub fn compile_wrapp_to_object(
    wasm: &[u8],
    object_file_path: &std::path::Path,
    target: crate::Target,
    cache: Option<&std::path::PathBuf>,
    is_pic: bool,
    export_dynamic: bool,
) -> anyhow::Result<()> {
    let mut config = webrogue_wasmtime_common::aot_config();
    config.target(target.name())?;
    config.cranelift_opt_level(wasmtime::OptLevel::SpeedAndSize);
    config.cranelift_regalloc_algorithm(wasmtime::RegallocAlgorithm::Backtracking);
    if let Some(cache) = cache {
        config.cache(Some(Cache::from_file(Some(cache))?));
    }
    if is_pic {
        unsafe {
            config.cranelift_flag_enable("is_pic");
        }
    }
    let engine = wasmtime::Engine::new(&config)?;

    let cwasm = engine.precompile_component(&wasm)?;

    let cwasm_info = crate::cwasm_analyzer::analyze_cwasm(&cwasm)?;

    let mut obj = object::write::Object::new(target.format(), target.arch(), target.endianness());

    obj.add_file_symbol(b"/main.wasm".into());
    let mut main_data = Vec::new();
    main_data.extend_from_slice(&(cwasm.len() as u64).to_le_bytes());
    main_data.extend_from_slice(&(cwasm_info.max_alignment as u64).to_le_bytes());
    main_data.extend_from_slice(&[0u8].repeat(cwasm_info.max_alignment as usize - 16));
    main_data.extend_from_slice(&cwasm);

    let main_symbol = obj.add_symbol(object::write::Symbol {
        name: b"WEBROGUE_AOT".into(),
        value: 0,
        size: 0,
        kind: object::SymbolKind::Text,
        scope: if export_dynamic {
            object::SymbolScope::Dynamic
        } else {
            object::SymbolScope::Linkage
        },
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

    let mut writer = std::io::BufWriter::new(std::fs::File::create(object_file_path)?);
    writer.write_all(&obj.write()?)?;
    std::io::Write::flush(&mut writer)?;

    Ok(())
}
