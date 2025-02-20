#[derive(Debug)]
pub struct NativeHandle(pub *const ());

unsafe impl Sync for NativeHandle {}
unsafe impl Send for NativeHandle {}

extern "C" {
    pub fn webrogue_gfx_ffi_create_system() -> *const ();
    pub fn webrogue_gfx_ffi_destroy_system(system_ptr: *const ());
    pub fn webrogue_gfx_ffi_create_window(system_ptr: *const ()) -> *const ();
    pub fn webrogue_gfx_ffi_destroy_window(window_ptr: *const ());
    pub fn webrogue_gfx_ffi_gl_get_proc_address(
        system_ptr: *const (),
        procname: *const std::ffi::c_char,
    ) -> *const ();
    pub fn webrogue_gfx_ffi_get_window_size(
        window_ptr: *const (),
        out_width: *mut u32,
        out_height: *mut u32,
    );
    pub fn webrogue_gfx_ffi_get_gl_size(
        window_ptr: *const (),
        out_width: *mut u32,
        out_height: *mut u32,
    );
    pub fn webrogue_gfx_ffi_present_window(window_ptr: *const ());
    pub fn webrogue_gfx_ffi_gl_init(window_ptr: *const ());
    pub fn webrogue_gfx_ffi_gl_commit_buffer(window_ptr: *const (), buf: *const (), len: u32);
    pub fn webrogue_gfx_ffi_gl_ret_buffer_read(window_ptr: *const (), buf: *mut (), len: u32);
    
}

// for ios DispatchQueue
// extern "C" {
//     fn webrogueRunOnMainThread<'a>(
//         f: extern "C" fn(userdata: *mut Box<dyn FnMut()>),
//         userdata: *mut Box<(dyn FnMut() + 'a)>,
//     );
// }

// for ios DispatchQueue
// extern "C" fn box_runner(userdata_ptr: *mut Box<dyn FnMut()>) {
//     // maybe unsafe cz it is actually FnOnce
//     unsafe {
//         // let f2 = userdata.as_ref().unwrap();
//         // let f3 = f2.as_ref();
//         // f3();
//         (*userdata_ptr)()
//     };
// }

// // for ios DispatchQueue
// #[inline]
// fn run_on_main_thread<T>(mut f: impl FnMut() -> T) -> T {
//     return f();
//     // let mut result: MaybeUninit<T> = MaybeUninit::uninit();
//     // let result_ptr: *mut MaybeUninit<T> = &mut result;
//     // let mut userdata: Box<dyn FnMut()> = Box::new(|| unsafe {
//     //     result_ptr.as_mut().unwrap().write(f());
//     // });
//     // let userdata_ptr: *mut Box<dyn FnMut()> = &mut userdata;
//     // unsafe { webrogueRunOnMainThread(box_runner, userdata_ptr) };
//     // drop(userdata);
//     // return unsafe { result.assume_init() };
// }
