use std::{cell::RefCell, rc::Rc, str::FromStr as _, sync::Arc};

use web_sys::js_sys::{JsString, Map, Object};

#[derive(Clone)]
pub struct DeferredLinker<ContextT> {
    funcs: Vec<
        Arc<
            dyn Fn(&mut Linker, Rc<RefCell<ContextT>>, Rc<dyn Fn() -> wiggle::GuestMemory<'static>>)
                + Send,
        >,
    >,
}

impl<ContextT> DeferredLinker<ContextT> {
    pub fn new() -> Self {
        Self { funcs: Vec::new() }
    }

    pub fn defer(
        &mut self,
        func: impl Fn(&mut Linker, Rc<RefCell<ContextT>>, Rc<dyn Fn() -> wiggle::GuestMemory<'static>>)
            + Send
            + 'static,
    ) {
        self.funcs.push(Arc::new(func));
    }

    pub fn into_linker(
        self,
        context: Rc<RefCell<ContextT>>,
        mem_fn: Rc<dyn Fn() -> wiggle::GuestMemory<'static>>,
    ) -> Linker {
        let mut linker = Linker::new();
        for func in self.funcs {
            func(&mut linker, context.clone(), mem_fn.clone());
        }
        linker
    }
}

pub struct Linker {
    map: Map<JsString, Map<JsString, wasm_bindgen::JsValue>>,
}

impl Linker {
    pub fn new() -> Self {
        Self {
            map: Map::new_typed(),
        }
    }

    pub fn add_import(&mut self, module: &str, name: &str, value: &wasm_bindgen::JsValue) {
        let module = JsString::from_str(module).unwrap();
        let name = JsString::from_str(name).unwrap();

        let module_map = match self.map.get_checked(&module) {
            Some(module_map) => module_map,
            None => {
                let module_map = Map::new_typed();
                self.map.set(&module, &module_map);
                module_map
            }
        };
        module_map.set(&name, value);
    }

    pub fn into_import_object(&self) -> Object {
        let module_map = Map::new_typed();
        for key in self.map.keys() {
            let key = key.unwrap();
            let module = self.map.get(&key);
            let module = Object::from_entries(&module).unwrap();
            module_map.set(&key, &module);
        }
        Object::from_entries(&module_map).unwrap()
    }
}
