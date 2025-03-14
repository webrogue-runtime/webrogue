extern "C" {
    static WEBROGUE_AOT: u8;
    // static WASMER_METADATA_WR_AOT_SIZE: usize;
}

pub fn aot_data() -> &'static [u8] {
    unsafe {
        let mut raw_bytes: [u8; 8] = [0; 8];
        std::ptr::copy_nonoverlapping((&WEBROGUE_AOT) as *const u8, raw_bytes.as_mut_ptr(), 8);
        let len = u64::from_le_bytes(raw_bytes);

        let mut raw_bytes: [u8; 8] = [0; 8];
        std::ptr::copy_nonoverlapping(
            ((&WEBROGUE_AOT) as *const u8).offset(8),
            raw_bytes.as_mut_ptr(),
            8,
        );
        let page_size = u64::from_le_bytes(raw_bytes);

        &std::slice::from_raw_parts(&WEBROGUE_AOT, (len + page_size) as usize)
            [(page_size as usize)..]
    }
}
