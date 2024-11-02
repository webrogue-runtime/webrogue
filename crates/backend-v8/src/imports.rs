pub struct Imports {
    pub f: Box<dyn Fn(&mut v8::HandleScope<v8::Context>, v8::Local<v8::Object>)>,
}

pub fn register_import(
    scope: &mut v8::HandleScope<v8::Context>,
    imports: v8::Local<v8::Object>,
    module_name: &str,
    value_name: &str,
    value: v8::Local<v8::Value>,
) {
    let module_name = v8::String::new(scope, module_name).unwrap();
    let module_object = imports.get(scope, module_name.into());
    let module_object = match module_object {
        Some(module_object) => {
            if module_object.is_object() {
                module_object.to_object(scope).unwrap()
            } else {
                v8::Object::new(scope)
            }
        }
        None => v8::Object::new(scope),
    };
    imports.set(scope, module_name.into(), module_object.into());
    let value_name = v8::String::new(scope, value_name).unwrap();
    module_object.set(scope, value_name.into(), value);
}
