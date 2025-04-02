extern "C" {
    pub fn wr_init_wasm_module(
        context: *mut std::ffi::c_void,
        json_ptr: *const u8,
        pointer: *const u8,
        size: u32,
    );
    // pub fn wr_reset_wasm();
    pub fn wr_exec_func(funcNamePtr: *const u8);
    pub fn wr_exec_func_ii(funcNamePtr: *const u8, arg0: i32, arg1: i32);
    // pub fn wr_error_size() -> u32;
    // pub fn wr_error_data(error: *mut u8);
    pub fn wr_make_shared_memory(initial_pages: u32, max_pages: u32);
}

pub struct ArgGetter<T> {
    _phantom_data: std::marker::PhantomData<T>,
}
macro_rules! get_arg {
    ($ffi_name:ident, $ty:ident) => {
        extern "C" {
            fn $ffi_name(argNum: u32) -> $ty;
        }
        impl ArgGetter<$ty> {
            #[allow(dead_code)]
            pub fn get(arg_num: u32) -> $ty {
                unsafe { $ffi_name(arg_num) }
            }
        }
    };
}
get_arg! {wr_get_arg_int32_t, i32}
get_arg! {wr_get_arg_int64_t, i64}
get_arg! {wr_get_arg_uint32_t, u32}
get_arg! {wr_get_arg_uint64_t, u64}
get_arg! {wr_get_arg_float, f32}
get_arg! {wr_get_arg_double, f64}

pub struct RetSetter<T> {
    _phantom_data: std::marker::PhantomData<T>,
}
macro_rules! set_ret {
    ($ffi_name:ident, $ty:ident) => {
        extern "C" {
            fn $ffi_name(val: $ty);
        }
        impl RetSetter<$ty> {
            #[allow(dead_code)]
            pub fn set(val: $ty) {
                unsafe { $ffi_name(val) }
            }
        }
    };
}
set_ret! {wr_set_ret_int32_t, i32}
set_ret! {wr_set_ret_int64_t, i64}
set_ret! {wr_set_ret_uint32_t, u32}
set_ret! {wr_set_ret_uint64_t, u64}
set_ret! {wr_set_ret_float, f32}
set_ret! {wr_set_ret_double, f64}

impl RetSetter<()> {
    pub fn set(_val: ()) {}
}
