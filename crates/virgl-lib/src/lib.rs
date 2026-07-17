extern "C" {
    pub fn webrogue_virgl_stub_fn();
}
pub fn stub_fn() {
    unsafe {
        webrogue_virgl_stub_fn();
    }
}
