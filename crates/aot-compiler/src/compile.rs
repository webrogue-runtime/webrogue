use std::collections::BTreeMap;
use wasmtime_environ::obj::LibCall;

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
        } else {
            config.cranelift_flag_enable("use_colocated_libcalls");
        }
    }
    let engine = wasmtime::Engine::new(&config)?;

    let wrapp_handle =
        webrogue_wrapp::WrappHandleBuilder::from_file_path(wrapp_file_path)?.build()?;

    let mut file = wrapp_handle
        .open_file("/app/main.wasm")
        .ok_or(anyhow::anyhow!("/app/main.wasm not found"))?;
    let mut wasm_binary = Vec::new();
    std::io::Read::read_to_end(&mut file, &mut wasm_binary)?;
    drop(file);

    let cwasm = engine.precompile_module(&wasm_binary)?;

    let cwasm_info = crate::cwasm_analyzer::analyze_cwasm(&cwasm, target, is_pic)?;

    let mut libcall_relocs: BTreeMap<String, Vec<u64>> = BTreeMap::new();

    for (offset, libcall) in cwasm_info.relocations.iter() {
        let offset = (cwasm_info.text.start + (*offset)) as u64;

        let libcall_name = match libcall {
            LibCall::FloorF32 => "floorf32",
            LibCall::FloorF64 => "floorf64",
            LibCall::NearestF32 => "nearestf32",
            LibCall::NearestF64 => "nearestf64",
            LibCall::CeilF32 => "ceilf32",
            LibCall::CeilF64 => "ceilf64",
            LibCall::TruncF32 => "truncf32",
            LibCall::TruncF64 => "truncf64",
            LibCall::FmaF32 => "fmaf32",
            LibCall::FmaF64 => "fmaf64",
            LibCall::X86Pshufb => "x86_pshufb",
        };
        let libcall_name = format!("wasmtime_libcall_{}", libcall_name);

        if let Some(value) = libcall_relocs.get_mut(&libcall_name) {
            value.push(offset as u64);
        } else {
            libcall_relocs.insert(libcall_name, vec![offset as u64]);
        };
    }

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

    for (libcall_name, offsets) in libcall_relocs.iter() {
        // External symbol for puts.
        let libcall_symbol = obj.add_symbol(object::write::Symbol {
            name: libcall_name.as_bytes().into(),
            value: 0,
            size: 0,
            kind: object::SymbolKind::Text,
            scope: object::SymbolScope::Dynamic,
            weak: false,
            section: object::write::SymbolSection::Undefined,
            flags: object::SymbolFlags::None,
        });

        for offset in offsets.iter() {
            obj.add_relocation(
                main_section,
                object::write::Relocation {
                    offset: *offset + cwasm_info.max_alignment,
                    symbol: libcall_symbol,
                    addend: target.reloc_append(is_pic),
                    flags: target.reloc_flags(is_pic),
                },
            )?;
        }
    }

    let mut writer = std::io::BufWriter::new(std::fs::File::create(&object_file_path)?);
    obj.write_stream(&mut writer)
        .map_err(|err| anyhow::anyhow!(err.to_string()))?;
    std::io::Write::flush(&mut writer)?;

    Ok(())
}
