use std::sync::Arc;

wai_bindgen_wasmer::export! { "webrogue-gfx.wai" }

extern "C" {
    fn webrogue_gfx_ffi_create_system() -> *const ();
    fn webrogue_gfx_ffi_destroy_system(system_ptr: *const ());
    fn webrogue_gfx_ffi_create_window(system_ptr: *const ()) -> *const ();
    fn webrogue_gfx_ffi_destroy_window(window_ptr: *const ());
    fn webrogue_gfx_ffi_gl_get_proc_address(
        system_ptr: *const (),
        procname: *const std::ffi::c_char,
    ) -> *const ();
    fn webrogue_gfx_ffi_get_window_size(
        window_ptr: *const (),
        out_width: *mut u32,
        out_height: *mut u32,
    );
    fn webrogue_gfx_ffi_get_gl_size(
        window_ptr: *const (),
        out_width: *mut u32,
        out_height: *mut u32,
    );
    fn webrogue_gfx_ffi_present_window(window_ptr: *const ());
}

struct NativeHandle(*const ());

unsafe impl Sync for NativeHandle {}
unsafe impl Send for NativeHandle {}

pub struct GFXInterface {
    gfx: Arc<GFX>,
    window: Option<Window>,
}

impl GFXInterface {
    pub fn new(gfx: Arc<GFX>) -> Self {
        #[cfg(feature = "fallback")]
        webrogue_gfx_fallback::dummy();
        Self { gfx, window: None }
    }

    pub fn add_to_imports(
        self,
        store: &mut wasmer::Store,
        imports: &mut wasmer::Imports,
    ) -> impl FnOnce(&wasmer::Instance, &dyn wasmer::AsStoreRef) -> Result<(), anyhow::Error> {
        webrogue_gfx::add_to_imports(store, imports, self)
    }
}

impl webrogue_gfx::WebrogueGfx for GFXInterface {
    fn make_window(&mut self) -> () {
        self.window = Some(self.gfx.make_window());
    }

    fn present(&mut self) -> () {
        self.window.as_mut().inspect(|window| {
            window.present();
        });
    }

    fn get_window_width(&mut self) -> u32 {
        self.window
            .as_ref()
            .and_then(|window| Some(window.get_size().0))
            .unwrap_or_default()
    }

    fn get_window_height(&mut self) -> u32 {
        self.window
            .as_ref()
            .and_then(|window| Some(window.get_size().1))
            .unwrap_or_default()
    }

    fn get_gl_width(&mut self) -> u32 {
        self.window
            .as_ref()
            .and_then(|window| Some(window.get_gl_size().0))
            .unwrap_or_default()
    }

    fn get_gl_height(&mut self) -> u32 {
        self.window
            .as_ref()
            .and_then(|window| Some(window.get_gl_size().1))
            .unwrap_or_default()
    }
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

pub struct GFX(NativeHandle);

impl Drop for GFX {
    fn drop(&mut self) {
        unsafe { webrogue_gfx_ffi_destroy_system(self.0 .0) }
    }
}
impl GFX {
    pub fn new() -> Self {
        Self {
            0: NativeHandle {
                0: unsafe { webrogue_gfx_ffi_create_system() },
            },
        }
    }

    fn make_window(&self) -> Window {
        Window {
            0: NativeHandle {
                0: unsafe { webrogue_gfx_ffi_create_window(self.0 .0) },
            },
        }
    }

    pub fn gl_get_proc_address(&self, procname: &str) -> *const () {
        let c_string = std::ffi::CString::new(procname).unwrap();
        unsafe { webrogue_gfx_ffi_gl_get_proc_address(self.0 .0, c_string.as_ptr()) }
    }
}
struct Window(NativeHandle);
impl Window {
    fn get_size(&self) -> (u32, u32) {
        let mut out: std::mem::MaybeUninit<(u32, u32)> = std::mem::MaybeUninit::uninit();
        unsafe {
            webrogue_gfx_ffi_get_window_size(
                self.0 .0,
                &mut out.assume_init_mut().0,
                &mut out.assume_init_mut().1,
            );
            out.assume_init()
        }
    }
    fn get_gl_size(&self) -> (u32, u32) {
        let mut out: std::mem::MaybeUninit<(u32, u32)> = std::mem::MaybeUninit::uninit();
        unsafe {
            webrogue_gfx_ffi_get_gl_size(
                self.0 .0,
                &mut out.assume_init_mut().0,
                &mut out.assume_init_mut().1,
            );
            out.assume_init()
        }
    }
    fn present(&self) {
        unsafe { webrogue_gfx_ffi_present_window(self.0 .0) }
    }
}
impl Drop for Window {
    fn drop(&mut self) {
        unsafe { webrogue_gfx_ffi_destroy_window(self.0 .0) }
    }
}
