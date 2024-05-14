use std::ffi::CString;
use std::ffi::NulError;
use std::os::raw::c_char;

#[macro_use]
extern crate lazy_static;

extern "C" {
    fn addLangExample(name: *const c_char, func: unsafe extern "C" fn());
    fn langExampleReturn(name: *const c_char);
    // fn webrogue_core_print(s: *const c_char);
}

lazy_static! {
    static ref GLOBAL_DATA: Result<CString, NulError> = CString::new("Hello from Rust!5");
}

extern "C" fn lang_example_rust() {
    unsafe {
        langExampleReturn(GLOBAL_DATA.as_ref().expect("CString::new failed").as_ptr());
    }
}

#[no_mangle]
pub extern "C" fn init_mod_langExampleRust() {
    unsafe {
        // let c_to_print = CString::new("Hello, world!").expect("CString::new failed");
        // webrogue_core_print(c_to_print.as_ptr());
        let name = CString::new("Rust language").expect("CString::new failed");
        addLangExample(name.as_ptr(), lang_example_rust);
    }
}
