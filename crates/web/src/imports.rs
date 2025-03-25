use core::sync;

pub struct Imports {
    pub initializers: Vec<fn(&mut Imports)>,
    pub funcs: Vec<Box<dyn Fn(&mut crate::context::Store)>>,
    pub config:
        std::collections::BTreeMap<&'static str, std::collections::BTreeMap<&'static str, u32>>,
}

unsafe impl Sync for Imports {}
unsafe impl Send for Imports {}

impl Clone for Imports {
    fn clone(&self) -> Self {
        let mut new_imports = Self {
            initializers: self.initializers.clone(),
            funcs: Vec::new(),
            config: self.config.clone(),
        };
        for initializer in self.initializers.iter() {
            (initializer)(&mut new_imports);
        }
        new_imports
    }
}

impl Imports {
    pub fn new() -> Self {
        Self {
            initializers: Vec::new(),
            funcs: Vec::new(),
            config: std::collections::BTreeMap::new(),
        }
    }

    pub fn add_initialozer(&mut self, initializer: fn(&mut Imports)) {
        self.initializers.push(initializer);
        (initializer)(self);
    }

    pub fn add_fn(
        &mut self,
        module_name: &'static str,
        func_name: &'static str,
        f: Box<dyn Fn(&mut crate::context::Store)>,
    ) {
        if !self.config.contains_key(module_name) {
            self.config
                .insert(module_name, std::collections::BTreeMap::new());
        }
        let module = self.config.get_mut(&module_name).unwrap();
        module.insert(func_name, self.funcs.len() as u32);
        self.funcs.push(f);
    }

    pub fn to_json(&self) -> String {
        let mut config_str = "{".to_owned();
        for (module_i, (module_name, module)) in self.config.iter().enumerate() {
            if module_i != 0 {
                config_str += ",";
            }
            config_str += &format!("\n    \"{}\": {{", module_name);
            for (func_i, (func_name, func_id)) in module.iter().enumerate() {
                if func_i != 0 {
                    config_str += ",";
                }

                config_str += &format!("\n        \"{}\": {{", func_name);
                // config_str += &format!("\n            \"ret_type\": \"{}\",", ret_type,);
                config_str += &format!("\n            \"func_id\": {}", func_id);
                config_str += "\n        }";
            }
            config_str += "\n    }";
        }
        config_str += "\n}";
        config_str
    }
}
