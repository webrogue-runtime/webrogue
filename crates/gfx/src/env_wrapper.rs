// #[derive(Clone)]
pub struct EnvWrapper {
    pub data: crate::interface::GFXInterface,
}
unsafe impl Send for EnvWrapper {}

unsafe impl Sync for EnvWrapper {}
