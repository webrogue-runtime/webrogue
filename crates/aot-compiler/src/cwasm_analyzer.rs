use anyhow::Context as _;
use object::{Object as _, ObjectSection as _};

pub struct CWasmInfo {
    pub max_alignment: u64,
}

pub fn analyze_cwasm(cwasm: &[u8]) -> anyhow::Result<CWasmInfo> {
    let mut max_alignment = 0;
    let obj = object::read::elf::ElfFile64::<object::Endianness>::parse(cwasm.into())
        .with_context(|| "failed to parse internal compilation artifact")?;

    for section in obj.sections() {
        max_alignment = std::cmp::max(section.align(), max_alignment);
    }
    Ok(CWasmInfo { max_alignment })
}
