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
}

pub fn analyze_cwasm(cwasm: &[u8]) -> anyhow::Result<CWasmInfo> {
    let mut max_alignment = 0;
    let obj = object::read::elf::ElfFile64::<object::Endianness>::parse(cwasm.into())
        .map_err(wasmtime_environ::obj::ObjectCrateErrorWrapper)
        .with_context(|| "failed to parse internal compilation artifact")?;

    for section in obj.sections() {
        max_alignment = std::cmp::max(section.align(), max_alignment);
    }
    Ok(CWasmInfo { max_alignment })
}
