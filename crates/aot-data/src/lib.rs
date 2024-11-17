extern "C" {
    static WASMER_METADATA_WR_AOT: u8;
    static WASMER_METADATA_WR_AOT_SIZE: usize;
}

pub fn aot_data() -> &'static [u8] {
    unsafe { std::slice::from_raw_parts(&WASMER_METADATA_WR_AOT, WASMER_METADATA_WR_AOT_SIZE) }
}
