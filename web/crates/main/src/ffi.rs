extern "C" {
    pub fn wr_rs_em_js_getArgU32(argNum: u32) -> u32;
    pub fn wr_rs_em_js_setResultU32(result: u32);
    pub fn wr_rs_em_js_getArgU64(argNum: u32) -> u64;
    pub fn wr_rs_em_js_setResultU64(result: u64);
    pub fn wr_rs_em_js_getArgI32(argNum: u32) -> i32;
    pub fn wr_rs_em_js_setResultI32(result: i32);
    pub fn wr_rs_em_js_getArgI64(argNum: u32) -> i64;
    pub fn wr_rs_em_js_setResultI64(result: i64);
    pub fn wr_rs_em_js_getArgF32(argNum: u32) -> f32;
    pub fn wr_rs_em_js_setResultF32(result: f32);
    pub fn wr_rs_em_js_getArgF64(argNum: u32) -> f64;
    pub fn wr_rs_em_js_setResultF64(result: f64);

    pub fn wr_rs_em_js_initWasmModule(
        context: *mut std::ffi::c_void,
        json_ptr: *const u8,
        pointer: *const u8,
        size: u32,
    );
    pub fn wr_rs_em_js_resetWasm();
    pub fn wr_rs_em_js_execFunc(funcNamePtr: *const u8);
    pub fn wr_rs_em_js_modErrorSize() -> u32;
    pub fn wr_rs_em_js_getModError(error: *mut u8);
    pub fn wr_rs_em_js_makeSharedMemory(inital_pages: u32, max_pages: u32);
}
