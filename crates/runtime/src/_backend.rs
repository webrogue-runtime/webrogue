use crate::runtime::Runtime;

pub trait Backend<Imports> {
    fn make_runtime(&self) -> Box<dyn Runtime<Imports>>;
}
