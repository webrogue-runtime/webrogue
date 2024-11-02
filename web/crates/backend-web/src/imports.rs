pub struct Imports {
    config: std::collections::BTreeMap<
        &'static str,
        std::collections::BTreeMap<&'static str, (u32, &'static str)>,
    >,
    pub funcs: Vec<Box<dyn Fn(&mut crate::Context)>>,
}

impl Imports {
    pub fn new() -> Self {
        Imports {
            config: std::collections::BTreeMap::new(),
            funcs: Vec::new(),
        }
    }
    pub fn add_fn(
        &mut self,
        module_name: &'static str,
        func_name: &'static str,
        ret_type: &'static str,
        f: Box<dyn Fn(&mut crate::Context)>,
    ) {
        if !self.config.contains_key(&module_name) {
            self.config
                .insert(module_name, std::collections::BTreeMap::new());
        }
        let module = self.config.get_mut(&module_name).unwrap();
        module.insert(func_name, (self.funcs.len() as u32, ret_type));
        self.funcs.push(f);
    }
    pub fn to_json(&self) -> String {
        let mut config_str = "{".to_owned();
        for (module_i, (module_name, module)) in self.config.iter().enumerate() {
            if module_i != 0 {
                config_str += ",";
            }
            config_str += &format!("\n    \"{}\": {{", module_name);
            for (func_i, (func_name, (func_id, ret_type))) in module.iter().enumerate() {
                if func_i != 0 {
                    config_str += ",";
                }

                config_str += &format!("\n        \"{}\": {{", func_name);
                config_str += &format!("\n            \"ret_type\": \"{}\",", ret_type,);
                config_str += &format!("\n            \"func_id\": {}", func_id);
                config_str += "\n        }";
            }
            config_str += "\n    }";
        }
        config_str += "\n}";
        config_str
    }
}
