pub fn add_bindings(
    exports: &mut wasmer::Exports,
    mut store: wasmer::StoreMut<'_>,
    env: wasmer::FunctionEnv<crate::env_wrapper::EnvWrapper>,
) {
    exports.insert("make-window",wasmer::Function::new_typed_with_env(&mut store, &env,move|mut store:wasmer::FunctionEnvMut<crate::env_wrapper::EnvWrapper>|  -> Result<(),wasmer::RuntimeError>{
            let data_mut = store.data_mut();
            let host =  &mut data_mut.data;
            let result = host.make_window();
            let() = result;
            Ok(())
        }));
    exports.insert("present",wasmer::Function::new_typed_with_env(&mut store, &env,move|mut store:wasmer::FunctionEnvMut<crate::env_wrapper::EnvWrapper>|  -> Result<(),wasmer::RuntimeError>{
            let data_mut = store.data_mut();
            let host =  &mut data_mut.data;
            let result = host.present();
            let() = result;
            Ok(())
        }));
    exports.insert("get-window-width",wasmer::Function::new_typed_with_env(&mut store, &env,move|mut store:wasmer::FunctionEnvMut<crate::env_wrapper::EnvWrapper>|  -> Result<i32,wasmer::RuntimeError>{
            let data_mut = store.data_mut();
            let host =  &mut data_mut.data;
            let result = host.get_window_width();
            Ok(result as i32)
        }));
    exports.insert("get-window-height",wasmer::Function::new_typed_with_env(&mut store, &env,move|mut store:wasmer::FunctionEnvMut<crate::env_wrapper::EnvWrapper>|  -> Result<i32,wasmer::RuntimeError>{
            let data_mut = store.data_mut();
            let host =  &mut data_mut.data;
            let result = host.get_window_height();
            Ok(result as i32)
        }));
    exports.insert("get-gl-width",wasmer::Function::new_typed_with_env(&mut store, &env,move|mut store:wasmer::FunctionEnvMut<crate::env_wrapper::EnvWrapper>|  -> Result<i32,wasmer::RuntimeError>{
            let data_mut = store.data_mut();
            let host =  &mut data_mut.data;
            let result = host.get_gl_width();
            Ok(result as i32)
        }));
    exports.insert("get-gl-height",wasmer::Function::new_typed_with_env(&mut store, &env,move|mut store:wasmer::FunctionEnvMut<crate::env_wrapper::EnvWrapper>|  -> Result<i32,wasmer::RuntimeError>{
            let data_mut = store.data_mut();
            let host =  &mut data_mut.data;
            let result = host.get_gl_height();
            Ok(result as i32)
        }));
}
