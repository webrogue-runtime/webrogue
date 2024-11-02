pub struct Imports {
    pub f: Box<
        dyn Fn(&mut wasmer::Imports, &mut wasmer::Store, wasmer::FunctionEnv<crate::context::Env>),
    >,
}
