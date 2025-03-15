use anyhow::Context as _;
use object::{Object as _, ObjectSection as _, ObjectSymbol as _};

fn subslice_range(inner: &[u8], outer: &[u8]) -> std::ops::Range<usize> {
    if inner.len() == 0 {
        return 0..0;
    }

    assert!(outer.as_ptr() <= inner.as_ptr());
    assert!((&inner[inner.len() - 1] as *const _) <= (&outer[outer.len() - 1] as *const _));

    let start = inner.as_ptr() as usize - outer.as_ptr() as usize;
    start..start + inner.len()
}

pub struct CWasmInfo {
    pub max_alignment: u64,
    pub relocations: Vec<(usize, wasmtime_environ::obj::LibCall)>,
    pub text: std::ops::Range<usize>,
}

// Copied from wasmtime::CodeMemory::new, so it must be kept in sync with original code
pub fn analyze_cwasm(
    cwasm: &[u8],
    target: crate::Target,
    is_pic: bool,
) -> anyhow::Result<CWasmInfo> {
    let mut max_alignment = 0;
    let obj = object::read::elf::ElfFile64::<object::Endianness>::parse(cwasm.into())
        .map_err(wasmtime_environ::obj::ObjectCrateErrorWrapper)
        .with_context(|| "failed to parse internal compilation artifact")?;

    let mut relocations = Vec::new();
    let mut text = 0..0;
    for section in obj.sections() {
        let data = section
            .data()
            .map_err(wasmtime_environ::obj::ObjectCrateErrorWrapper)?;
        let name = section
            .name()
            .map_err(wasmtime_environ::obj::ObjectCrateErrorWrapper)?;
        let range = subslice_range(data, cwasm);

        max_alignment = std::cmp::max(section.align(), max_alignment);
        if section.align() != 0 && data.len() != 0 {
            if (data.as_ptr() as u64 - cwasm.as_ptr() as u64) % section.align() != 0 {
                anyhow::bail!(
                    "section `{}` isn't aligned to {:#x}",
                    section.name().unwrap_or("ERROR"),
                    section.align()
                );
            }
        }

        match name {
            ".text" => {
                text = range;
                for (offset, reloc) in section.relocations() {
                    if reloc.kind() != target.reloc_kind(is_pic) {
                        anyhow::bail!(
                            "Expected reloc_kind: {:?}, but cwasm contains: {:?}",
                            target.reloc_kind(is_pic),
                            reloc.kind()
                        )
                    }
                    if reloc.encoding() != target.reloc_encoding(is_pic) {
                        anyhow::bail!(
                            "Expected reloc_encoding: {:?}, but cwasm contains: {:?}",
                            target.reloc_encoding(is_pic),
                            reloc.encoding()
                        )
                    }
                    if reloc.size() != target.reloc_size(is_pic) {
                        anyhow::bail!(
                            "Expected reloc_size: {:?}, but cwasm contains {:?}",
                            target.reloc_size(is_pic),
                            reloc.size()
                        )
                    }
                    if reloc.addend() != target.reloc_append(is_pic) {
                        anyhow::bail!(
                            "Expected reloc_append: {:?}, but cwasm contains {:?}",
                            target.reloc_append(is_pic),
                            reloc.addend()
                        )
                    }
                    assert_eq!(reloc.addend(), target.reloc_append(is_pic));
                    let sym = match reloc.target() {
                        object::RelocationTarget::Symbol(id) => id,
                        other => panic!("unknown relocation target {other:?}"),
                    };
                    let sym = obj.symbol_by_index(sym).unwrap().name().unwrap();
                    let libcall = wasmtime_environ::obj::LibCall::from_str(sym)
                        .unwrap_or_else(|| panic!("unknown symbol relocation: {sym}"));

                    let offset = usize::try_from(offset).unwrap();
                    relocations.push((offset, libcall));
                }
            }
            _ => {}
        };
    }
    Ok(CWasmInfo {
        max_alignment,
        relocations,
        text,
    })
}
