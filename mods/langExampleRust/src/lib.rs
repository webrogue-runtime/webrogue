use std::ffi::CString;
use std::os::raw::c_char;

// #[macro_use]
// extern crate lazy_static;

extern crate rand;
use rand::Rng;

extern "C" {
    fn addLangExample(name: *const c_char, func: unsafe extern "C" fn());
    fn langExampleReturn(name: *const c_char);
    // fn webrogue_core_print(s: *const c_char);
}

extern "C" fn lang_example_rust() {
    let mut rng = rand::thread_rng();
    let text = format!(
        "Hello from {}! random digit: {}",
        "Rust",
        rng.gen_range(0..10)
    );
    let c_text = CString::new(text).expect("CString::new failed");
    unsafe {
        langExampleReturn(c_text.as_ptr());
    }
}

#[no_mangle]
pub extern "C" fn init_mod_langExampleRust() {
    let name = CString::new("Rust language").expect("CString::new failed");
    unsafe {
        // let c_to_print = CString::new("Hello, world!").expect("CString::new failed");
        // webrogue_core_print(c_to_print.as_ptr());
        addLangExample(name.as_ptr(), lang_example_rust);
    }
}
