use crate::env_wrapper::EnvWrapper;
use crate::memory_handle::MemoryHandle;
use crate::{ffi, GL};

pub fn add_to_imports(
    exports: &mut wasmer::Exports,
    store: &mut wasmer::StoreMut<'_>,
    env: &wasmer::FunctionEnv<EnvWrapper>,
) {
    exports.insert(
        "init-ptrs",
        wasmer::Function::new_typed_with_env(
            store,
            env,
            move |mut store: wasmer::FunctionEnvMut<EnvWrapper>|
                  -> Result<(), wasmer::RuntimeError> {
                let mut gl = store.data_mut().gl.write().unwrap();
                let gfx = gl.gfx.clone();
                gl.proc_addresses.fill(gfx);
                Ok(())
            },
        ),
    );

    exports.insert(
        "glShaderSource",
        wasmer::Function::new_typed_with_env(
            store,
            env,
            move |mut store: wasmer::FunctionEnvMut<EnvWrapper>,
                  shader: u32,
                  count: i32,
                  string: u32,
                  length: u32|
                  -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.read().unwrap();
                let memory =
                    MemoryHandle::new(store.data().lazy.get().unwrap().memory.clone(), &store);

                let mut sources: Vec<Vec<i8>> = vec![];
                sources.reserve(count as usize);
                for i in 0..count {
                    let mut size = None;
                    if length != 0 {
                        let size_candidate = memory.read::<i32>(length as u64 + i as u64 * 4)?;
                        if size_candidate >= 0 {
                            size = Some(size_candidate);
                        }
                    }
                    let string_offset = memory.read::<i32>(string as u64 + i as u64 * 4)?;
                    sources.push(match size {
                        None => {
                            let mut source: Vec<i8> = vec![];
                            let mut byte_offset = 0;
                            loop {
                                let byte =
                                    memory.read::<i8>(string_offset as u64 + byte_offset as u64)?;
                                if byte == 0 {
                                    break;
                                } else {
                                    byte_offset += 1;
                                    source.push(byte);
                                }
                            }
                            source
                        }
                        Some(size) => memory.read_vec::<i8>(string_offset as u64, size as u64)?,
                    });
                }

                let mut converted_string: Vec<*const i8> = vec![];
                let mut converted_length: Vec<std::os::raw::c_int> = vec![];

                sources.iter_mut().for_each(|source| {
                    converted_string.push(source.as_ptr());
                    converted_length.push(source.len() as std::os::raw::c_int);
                });

                unsafe {
                    (gl.proc_addresses.glShaderSource)(
                        shader as std::os::raw::c_uint,
                        count as std::os::raw::c_int,
                        converted_string.as_ptr(),
                        converted_length.as_mut_ptr(),
                    )
                };
                drop(sources);
                Ok(())
            },
        ),
    );

    exports.insert(
        "glVertexAttribPointer",
        wasmer::Function::new_typed_with_env(
            store,
            env,
            move |mut store: wasmer::FunctionEnvMut<EnvWrapper>,
                  index: u32,
                  size: i32,
                  _type: u32,
                  normalized: u32,
                  stride: i32,
                  pointer: u32|
                  -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.read().unwrap();
                unsafe {
                    (gl.proc_addresses.glVertexAttribPointer)(
                        index,
                        size,
                        _type,
                        normalized as u8,
                        stride,
                        pointer as *mut (),
                    )
                };
                Ok(())
            },
        ),
    );

    exports.insert(
        "glDrawElements",
        wasmer::Function::new_typed_with_env(
            store,
            env,
            move |mut store: wasmer::FunctionEnvMut<EnvWrapper>,
                  mode: u32,
                  count: i32,
                  _type: u32,
                  indices: u32|
                  -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let mut gl = gl_arc.write().unwrap();
                let memory =
                    MemoryHandle::new(store.data().lazy.get().unwrap().memory.clone(), &store);
                let mut element_array_buffer = 0;
                unsafe {
                    (gl.proc_addresses.glGetIntegerv)(
                        ffi::GL_ELEMENT_ARRAY_BUFFER_BINDING,
                        &mut element_array_buffer,
                    );
                }
                if element_array_buffer == 0 {
                    let len_indices = (crate::compsize::glDrawElements_indices_compsize(
                        &mut gl,
                        count,
                        crate::ffi::GLEnumGroupDrawElementsType::from_raw(_type),
                    )) as usize;
                    let indices_cow = memory.read_vec::<u8>(indices as u64, len_indices as u64)?;
                    unsafe {
                        (gl.proc_addresses.glDrawElements)(
                            mode,
                            count,
                            _type,
                            indices_cow.as_ptr() as *const (),
                        )
                    };
                } else {
                    unsafe {
                        (gl.proc_addresses.glDrawElements)(mode, count, _type, indices as *const ())
                    };
                }
                Ok(())
            },
        ),
    );

    exports.insert(
        "glDrawElementsInstanced",
        wasmer::Function::new_typed_with_env(
            store,
            env,
            move |mut store: wasmer::FunctionEnvMut<EnvWrapper>,
                  mode: u32,
                  count: i32,
                  _type: u32,
                  indices: u32,
                  instancecount: i32|
                  -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let mut gl = gl_arc.write().unwrap();
                let memory =
                    MemoryHandle::new(store.data().lazy.get().unwrap().memory.clone(), &store);
                let mut element_array_buffer = 0;
                unsafe {
                    (gl.proc_addresses.glGetIntegerv)(
                        ffi::GL_ELEMENT_ARRAY_BUFFER_BINDING,
                        &mut element_array_buffer,
                    );
                }
                if element_array_buffer == 0 {
                    let len_indices = (crate::compsize::glDrawElements_indices_compsize(
                        &mut gl,
                        count,
                        crate::ffi::GLEnumGroupDrawElementsType::from_raw(_type),
                    )) as usize;
                    let indices_cow = memory.read_vec::<u8>(indices as u64, len_indices as u64)?;
                    unsafe {
                        (gl.proc_addresses.glDrawElementsInstanced)(
                            mode,
                            count,
                            _type,
                            indices_cow.as_ptr() as *const (),
                            instancecount,
                        )
                    };
                } else {
                    unsafe {
                        (gl.proc_addresses.glDrawElementsInstanced)(
                            mode,
                            count,
                            _type,
                            indices as *const (),
                            instancecount,
                        )
                    };
                }
                Ok(())
            },
        ),
    );

    exports.insert(
        "glDrawRangeElements",
        wasmer::Function::new_typed_with_env(
            store,
            env,
            move |mut store: wasmer::FunctionEnvMut<EnvWrapper>,
                  mode: u32,
                  start: u32,
                  end: u32,
                  count: i32,
                  _type: u32,
                  indices: u32|
                  -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let mut gl = gl_arc.write().unwrap();
                let memory =
                    MemoryHandle::new(store.data().lazy.get().unwrap().memory.clone(), &store);
                let mut element_array_buffer = 0;
                unsafe {
                    (gl.proc_addresses.glGetIntegerv)(
                        ffi::GL_ELEMENT_ARRAY_BUFFER_BINDING,
                        &mut element_array_buffer,
                    );
                }
                if element_array_buffer == 0 {
                    let len_indices = (crate::compsize::glDrawElements_indices_compsize(
                        &mut gl,
                        count,
                        crate::ffi::GLEnumGroupDrawElementsType::from_raw(_type),
                    )) as usize;
                    let indices_cow = memory.read_vec::<u8>(indices as u64, len_indices as u64)?;

                    unsafe {
                        (gl.proc_addresses.glDrawRangeElements)(
                            mode,
                            start,
                            end,
                            count,
                            _type,
                            indices_cow.as_ptr() as *const (),
                        )
                    };
                } else {
                    unsafe {
                        (gl.proc_addresses.glDrawRangeElements)(
                            mode,
                            start,
                            end,
                            count,
                            _type,
                            indices as *const (),
                        )
                    };
                }
                Ok(())
            },
        ),
    );
    // // TODO check
    exports.insert(
        "glGetUniformIndices",
        wasmer::Function::new_typed_with_env(
            store,
            env,
            move |mut store: wasmer::FunctionEnvMut<EnvWrapper>,
                  program: u32,
                  uniform_count: i32,
                  uniform_names: u32,
                  uniform_indices: u32|
                  -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.read().unwrap();
                let memory =
                    MemoryHandle::new(store.data().lazy.get().unwrap().memory.clone(), &store);
                let uniform_names_ptrs =
                    memory.read_vec::<u32>(uniform_names as u64, uniform_count as u64)?;
                let mut uniform_names: Vec<Vec<u8>> = Vec::new();
                for ptr in uniform_names_ptrs {
                    uniform_names.push(memory.read_vec::<u8>(
                        ptr as u64,
                        (crate::utils::guest_strlen(&memory, ptr as u64) + 1) as u64,
                    )?);
                }
                let converted_uniform_names = uniform_names
                    .iter()
                    .map(|name| name.as_ptr() as *const i8)
                    .collect::<Vec<_>>();

                let mut vec_uniform_indices: Vec<std::os::raw::c_uint> = vec![];
                vec_uniform_indices.reserve(uniform_count as usize);
                for i in 0..(uniform_count as u64) {
                    // TODO vec
                    vec_uniform_indices
                        .push(memory.read::<u32>(uniform_indices as u64 + i * 4)?
                            as std::os::raw::c_uint);
                }
                let converted_uniform_indices =
                    vec_uniform_indices.as_mut_ptr() as *mut std::os::raw::c_uint;
                let result = unsafe {
                    (gl.proc_addresses.glGetUniformIndices)(
                        program,
                        uniform_count,
                        converted_uniform_names.as_ptr(),
                        converted_uniform_indices,
                    )
                };
                memory.write_slice::<u32>(
                    uniform_indices as u64,
                    &vec_uniform_indices
                        .iter()
                        .map(|value| *value as u32)
                        .collect::<Vec<_>>(),
                )?;
                Ok(result)
            },
        ),
    );

    exports.insert(
        "glMapBufferRange",
        wasmer::Function::new_typed_with_env(
            store,
            env,
            move |mut _store: wasmer::FunctionEnvMut<EnvWrapper>,
                  _target: u32,
                  _offset: i32,
                  _length: i32,
                  _access: u32|
                  -> Result<i32, wasmer::RuntimeError> { todo!() },
        ),
    );
    exports.insert(
        "glTexImage2D",
        wasmer::Function::new_typed_with_env(
            store,
            env,
            move |mut store: wasmer::FunctionEnvMut<EnvWrapper>,
                  target: u32,
                  level: i32,
                  internalformat: i32,
                  width: i32,
                  height: i32,
                  border: i32,
                  format: u32,
                  _type: u32,
                  pixels: u32|
                  -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let mut gl = gl_arc.write().unwrap();
                let memory =
                    MemoryHandle::new(store.data().lazy.get().unwrap().memory.clone(), &store);
                let converted_target = target;
                let converted_level = level;
                let converted_internalformat = internalformat;
                let converted_width = width;
                let converted_height = height;
                let converted_border = border;
                let converted_format = format;
                let converted_type = _type;
                let len_pixels = (crate::compsize::glTexImage2D_pixels_compsize(
                    &mut gl,
                    crate::ffi::GLEnumGroupPixelFormat::from_raw(format),
                    crate::ffi::GLEnumGroupPixelType::from_raw(_type),
                    width,
                    height,
                )) as usize;

                let slice = if pixels != 0 {
                    Some(memory.read_vec::<u8>(pixels as u64, len_pixels as u64)?)
                } else {
                    None
                };

                let converted_pixels = match &slice {
                    Some(slice) => slice.as_ptr() as *mut (),
                    None => std::ptr::null_mut(),
                };

                unsafe {
                    (gl.proc_addresses.glTexImage2D)(
                        converted_target,
                        converted_level,
                        converted_internalformat,
                        converted_width,
                        converted_height,
                        converted_border,
                        converted_format,
                        converted_type,
                        converted_pixels,
                    )
                };
                Ok(())
            },
        ),
    );
    exports.insert(
        "glVertexAttribIPointer",
        wasmer::Function::new_typed_with_env(
            store,
            env,
            move |mut store: wasmer::FunctionEnvMut<EnvWrapper>,
                  index: u32,
                  size: i32,
                  _type: u32,
                  stride: i32,
                  pointer: u32|
                  -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.read().unwrap();

                unsafe {
                    (gl.proc_addresses.glVertexAttribIPointer)(
                        index,
                        size,
                        _type,
                        stride,
                        pointer as *mut (),
                    )
                };
                Ok(())
            },
        ),
    );

    exports.insert(
        "glGetStringData",
        wasmer::Function::new_typed_with_env(
            store,
            env,
            move |mut store: wasmer::FunctionEnvMut<EnvWrapper>,
                  i: i32,
                  name: u32,
                  data_ptr: u32|
                  -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let mut gl = gl_arc.write().unwrap();

                if let Some(s) = get_string_i(&mut gl, i, name) {
                    let memory =
                        MemoryHandle::new(store.data().lazy.get().unwrap().memory.clone(), &store);
                    memory.write_slice::<u8>(data_ptr as u64, &s)?;
                }
                Ok(())
            },
        ),
    );
    exports.insert(
        "glGetStringLen",
        wasmer::Function::new_typed_with_env(
            store,
            env,
            move |mut store: wasmer::FunctionEnvMut<EnvWrapper>,
                  i: i32,
                  name: u32|
                  -> Result<i32, wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let mut gl = gl_arc.write().unwrap();
                let result = match get_string_i(&mut gl, i, name) {
                    None => -1,
                    Some(s) => s.len() as i32,
                };
                Ok(result)
            },
        ),
    );
}

fn get_string(gl: &mut GL, name: u32) -> Option<Vec<u8>> {
    let gl_str = unsafe { (gl.proc_addresses.glGetString)(name) };
    if gl_str.is_null() {
        return None;
    }
    let owned_str = unsafe { std::ffi::CStr::from_ptr(gl_str as *const std::ffi::c_char) };
    if name == crate::ffi::GL_EXTENSIONS {
        let filtered_extensions = owned_str
            .to_str()
            .unwrap()
            .split(" ")
            .filter(|extension| ffi::EXTENSIONS.contains(extension))
            .collect::<Vec<_>>();
        let _unsupported_extensions = owned_str
            .to_str()
            .unwrap()
            .split(" ")
            .filter(|extension| !ffi::EXTENSIONS.contains(extension))
            .collect::<Vec<_>>();
        let mut result = (filtered_extensions.join(" ") + " ").into_bytes();
        result.push(0);
        return Some(result);
    }
    Some(owned_str.to_bytes_with_nul().to_vec())
}

fn get_string_i(gl: &mut GL, i: i32, name: u32) -> Option<Vec<u8>> {
    if i == -1 {
        get_string(gl, name)
    } else {
        // TODO check
        match get_string(gl, name) {
            None => None,
            Some(str) => {
                let mut result = vec![];
                let mut found = false;
                let mut current_i = 0;
                for c in str {
                    if current_i == i {
                        found = true;
                        result.push(c);
                    }
                    if c == b' ' {
                        if found {
                            return Some(result);
                        } else {
                            current_i += 1;
                        }
                    }
                }
                return None;
            }
        }
    }
}
