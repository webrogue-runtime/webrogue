use anyhow::Ok;

pub struct Runtime {}

impl Runtime {
    pub fn new() -> Self {
        Self {}
    }
}

extern "C" {
    fn wr_rs_em_js_initWasmModule(
        context: *mut std::ffi::c_void,
        json_ptr: *const u8,
        pointer: *const u8,
        size: u32,
    );
    fn wr_rs_em_js_resetWasm();
    fn wr_rs_em_js_execFunc(funcNamePtr: *const u8);
    fn wr_rs_em_js_modErrorSize() -> u32;
    fn wr_rs_em_js_getModError(error: *mut u8);
    fn wr_rs_em_js_makeSharedMemory(inital_pages: u32, max_pages: u32);
}

fn exec_func(func_name: &str) {
    unsafe {
        let mut func_name = func_name.as_bytes().to_vec();
        func_name.push(0);
        wr_rs_em_js_execFunc(func_name.as_ptr());
    }
}

#[no_mangle]
extern "C" fn wr_rs_exported_fn(func_i: u32, context: *mut std::ffi::c_void) {
    let context_ref = unsafe { (context as *mut crate::context::Context).as_mut().unwrap() };
    let func = &context_ref.imports.funcs[func_i as usize];
    func(context_ref);
}

#[no_mangle]
extern "C" fn wr_rs_exported_async_fn(func_i: u32, context: *mut std::ffi::c_void) {
    let context_ref = unsafe { (context as *mut crate::context::Context).as_mut().unwrap() };
    let func = &context_ref.imports.funcs[func_i as usize];
    func(context_ref);
}

impl webrogue_runtime::Runtime<crate::Imports> for Runtime {
    fn run(
        &self,
        imports: crate::Imports,
        context_vec: webrogue_runtime::ContextVec,
        bytecode: Vec<u8>,
        memory_size_range: Option<(u64, u64)>,
    ) -> anyhow::Result<()> {
        unsafe {
            if let Some(memory_size_range) = memory_size_range {
                wr_rs_em_js_makeSharedMemory(
                    memory_size_range.0 as u32,
                    memory_size_range.1 as u32,
                );
            }
            let mut worker_config = imports.to_json().as_bytes().to_vec();
            worker_config.push(0);
            let mut context = crate::context::Context {
                context_vec: context_vec,
                memory_factory: Box::new(crate::memory::MemoryFactory {}),
                imports: &imports,
            };

            wr_rs_em_js_initWasmModule(
                ((&mut context) as *mut crate::context::Context) as *mut std::ffi::c_void,
                worker_config.as_ptr(),
                bytecode.as_ptr(),
                bytecode.len() as u32,
            );

            exec_func("_start");
            //     let error_size = wr_rs_em_js_modErrorSize();
            // if error_size != 0 {
            //     let mut error_text = vec![0u8; error_size as usize];
            //     wr_rs_em_js_getModError(error_text.as_mut_ptr());
            //     eprintln!("error: {}", String::from_utf8(error_text).unwrap());
            //     return false;
            // }
            wr_rs_em_js_resetWasm();
        }
        Ok(())
    }
}
