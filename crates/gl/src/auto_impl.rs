use crate::{env_wrapper::EnvWrapper, memory_handle::MemoryHandle};

// DO NOT EDIT! This file is generated automatically

#[allow(non_snake_case)]
#[rustfmt::skip]
pub fn add_to_imports(
    exports: &mut wasmer::Exports,
    store: &mut wasmer::StoreMut<'_>,
    env: &wasmer::FunctionEnv<EnvWrapper>,
) {
    exports.insert(
    "glActiveTexture",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            texture: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let converted_texture = texture;
                let result = unsafe {
                    (gl.proc_addresses.glActiveTexture)(
                        converted_texture,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glAttachShader",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            program: u32,
            shader: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let converted_program = program;
                let converted_shader = shader;
                let result = unsafe {
                    (gl.proc_addresses.glAttachShader)(
                        converted_program,
                        converted_shader,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glBeginQuery",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            target: u32,
            id: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let converted_target = target;
                let converted_id = id;
                let result = unsafe {
                    (gl.proc_addresses.glBeginQuery)(
                        converted_target,
                        converted_id,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glBeginTransformFeedback",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            primitiveMode: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let converted_primitiveMode = primitiveMode;
                let result = unsafe {
                    (gl.proc_addresses.glBeginTransformFeedback)(
                        converted_primitiveMode,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glBindAttribLocation",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            program: u32,
            index: u32,
            name: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let memory =
                    MemoryHandle::new(store.data().lazy.get().unwrap().memory.clone(), &store);
                let converted_program = program;
                let converted_index = index;

                let len_name = (crate::utils::guest_strlen(&memory, name as u64) + 1) as usize;
                let mut vec_name: Vec<i8> = memory.read_vec::<i8>(name as u64, len_name as u64)?;
                let converted_name = vec_name.as_mut_ptr() as *const i8;

                let result = unsafe {
                    (gl.proc_addresses.glBindAttribLocation)(
                        converted_program,
                        converted_index,
                        converted_name,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glBindBuffer",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            target: u32,
            buffer: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let converted_target = target;
                let converted_buffer = buffer;
                let result = unsafe {
                    (gl.proc_addresses.glBindBuffer)(
                        converted_target,
                        converted_buffer,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glBindBufferBase",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            target: u32,
            index: u32,
            buffer: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let converted_target = target;
                let converted_index = index;
                let converted_buffer = buffer;
                let result = unsafe {
                    (gl.proc_addresses.glBindBufferBase)(
                        converted_target,
                        converted_index,
                        converted_buffer,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glBindBufferRange",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            target: u32,
            index: u32,
            buffer: u32,
            offset: i32,
            size: i32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let converted_target = target;
                let converted_index = index;
                let converted_buffer = buffer;
                let converted_offset = offset as isize;
                let converted_size = size as isize;
                let result = unsafe {
                    (gl.proc_addresses.glBindBufferRange)(
                        converted_target,
                        converted_index,
                        converted_buffer,
                        converted_offset,
                        converted_size,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glBindFramebuffer",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            target: u32,
            framebuffer: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let converted_target = target;
                let converted_framebuffer = framebuffer;
                let result = unsafe {
                    (gl.proc_addresses.glBindFramebuffer)(
                        converted_target,
                        converted_framebuffer,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glBindRenderbuffer",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            target: u32,
            renderbuffer: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let converted_target = target;
                let converted_renderbuffer = renderbuffer;
                let result = unsafe {
                    (gl.proc_addresses.glBindRenderbuffer)(
                        converted_target,
                        converted_renderbuffer,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glBindSampler",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            unit: u32,
            sampler: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let converted_unit = unit;
                let converted_sampler = sampler;
                let result = unsafe {
                    (gl.proc_addresses.glBindSampler)(
                        converted_unit,
                        converted_sampler,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glBindTexture",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            target: u32,
            texture: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let converted_target = target;
                let converted_texture = texture;
                let result = unsafe {
                    (gl.proc_addresses.glBindTexture)(
                        converted_target,
                        converted_texture,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glBindTransformFeedback",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            target: u32,
            id: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let converted_target = target;
                let converted_id = id;
                let result = unsafe {
                    (gl.proc_addresses.glBindTransformFeedback)(
                        converted_target,
                        converted_id,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glBindVertexArray",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            array: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let converted_array = array;
                let result = unsafe {
                    (gl.proc_addresses.glBindVertexArray)(
                        converted_array,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glBindVertexArrayOES",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            array: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let converted_array = array;
                let result = unsafe {
                    (gl.proc_addresses.glBindVertexArrayOES)(
                        converted_array,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glBlendColor",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            red: f32,
            green: f32,
            blue: f32,
            alpha: f32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let converted_red = red;
                let converted_green = green;
                let converted_blue = blue;
                let converted_alpha = alpha;
                let result = unsafe {
                    (gl.proc_addresses.glBlendColor)(
                        converted_red,
                        converted_green,
                        converted_blue,
                        converted_alpha,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glBlendEquation",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            mode: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let converted_mode = mode;
                let result = unsafe {
                    (gl.proc_addresses.glBlendEquation)(
                        converted_mode,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glBlendEquationSeparate",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            modeRGB: u32,
            modeAlpha: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let converted_modeRGB = modeRGB;
                let converted_modeAlpha = modeAlpha;
                let result = unsafe {
                    (gl.proc_addresses.glBlendEquationSeparate)(
                        converted_modeRGB,
                        converted_modeAlpha,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glBlendFunc",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            sfactor: u32,
            dfactor: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let converted_sfactor = sfactor;
                let converted_dfactor = dfactor;
                let result = unsafe {
                    (gl.proc_addresses.glBlendFunc)(
                        converted_sfactor,
                        converted_dfactor,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glBlendFuncSeparate",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            sfactorRGB: u32,
            dfactorRGB: u32,
            sfactorAlpha: u32,
            dfactorAlpha: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let converted_sfactorRGB = sfactorRGB;
                let converted_dfactorRGB = dfactorRGB;
                let converted_sfactorAlpha = sfactorAlpha;
                let converted_dfactorAlpha = dfactorAlpha;
                let result = unsafe {
                    (gl.proc_addresses.glBlendFuncSeparate)(
                        converted_sfactorRGB,
                        converted_dfactorRGB,
                        converted_sfactorAlpha,
                        converted_dfactorAlpha,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glBlitFramebuffer",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            srcX0: i32,
            srcY0: i32,
            srcX1: i32,
            srcY1: i32,
            dstX0: i32,
            dstY0: i32,
            dstX1: i32,
            dstY1: i32,
            mask: u32,
            filter: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let converted_srcX0 = srcX0;
                let converted_srcY0 = srcY0;
                let converted_srcX1 = srcX1;
                let converted_srcY1 = srcY1;
                let converted_dstX0 = dstX0;
                let converted_dstY0 = dstY0;
                let converted_dstX1 = dstX1;
                let converted_dstY1 = dstY1;
                let converted_mask = mask;
                let converted_filter = filter;
                let result = unsafe {
                    (gl.proc_addresses.glBlitFramebuffer)(
                        converted_srcX0,
                        converted_srcY0,
                        converted_srcX1,
                        converted_srcY1,
                        converted_dstX0,
                        converted_dstY0,
                        converted_dstX1,
                        converted_dstY1,
                        converted_mask,
                        converted_filter,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glBufferData",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            target: u32,
            size: i32,
            data: u32,
            usage: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let memory =
                    MemoryHandle::new(store.data().lazy.get().unwrap().memory.clone(), &store);
                let converted_target = target;
                let converted_size = size as isize;

                let len_data = (size) as usize;
                let mut vec_data: Vec<u8> = memory.read_vec::<u8>(data as u64, len_data as u64)?;
                let converted_data = vec_data.as_mut_ptr() as *const ();

                let converted_usage = usage;
                let result = unsafe {
                    (gl.proc_addresses.glBufferData)(
                        converted_target,
                        converted_size,
                        converted_data,
                        converted_usage,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glBufferSubData",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            target: u32,
            offset: i32,
            size: i32,
            data: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let memory =
                    MemoryHandle::new(store.data().lazy.get().unwrap().memory.clone(), &store);
                let converted_target = target;
                let converted_offset = offset as isize;
                let converted_size = size as isize;

                let len_data = (size) as usize;
                let mut vec_data: Vec<u8> = memory.read_vec::<u8>(data as u64, len_data as u64)?;
                let converted_data = vec_data.as_mut_ptr() as *const ();

                let result = unsafe {
                    (gl.proc_addresses.glBufferSubData)(
                        converted_target,
                        converted_offset,
                        converted_size,
                        converted_data,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glCheckFramebufferStatus",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            target: u32,
        | -> Result<u32, wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let converted_target = target;
                let result = unsafe {
                    (gl.proc_addresses.glCheckFramebufferStatus)(
                        converted_target,
                    ) 
                };

                Ok(result.into())
            },
        ),
    );
    exports.insert(
    "glClear",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            mask: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let converted_mask = mask;
                let result = unsafe {
                    (gl.proc_addresses.glClear)(
                        converted_mask,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glClearBufferfi",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            buffer: u32,
            drawbuffer: i32,
            depth: f32,
            stencil: i32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let converted_buffer = buffer;
                let converted_drawbuffer = drawbuffer;
                let converted_depth = depth;
                let converted_stencil = stencil;
                let result = unsafe {
                    (gl.proc_addresses.glClearBufferfi)(
                        converted_buffer,
                        converted_drawbuffer,
                        converted_depth,
                        converted_stencil,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glClearBufferfv",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            buffer: u32,
            drawbuffer: i32,
            value: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let mut gl = gl_arc.borrow_mut();
                let memory =
                    MemoryHandle::new(store.data().lazy.get().unwrap().memory.clone(), &store);
                let converted_buffer = buffer;
                let converted_drawbuffer = drawbuffer;

                let len_value = (crate::compsize::glClearBufferfv_value_compsize(&mut gl, crate::ffi::GLEnumGroupBuffer::from_raw(buffer))) as usize;
                let mut vec_value: Vec<f32> = memory.read_vec::<f32>(value as u64, len_value as u64)?;
                let converted_value = vec_value.as_mut_ptr() as *const f32;

                let result = unsafe {
                    (gl.proc_addresses.glClearBufferfv)(
                        converted_buffer,
                        converted_drawbuffer,
                        converted_value,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glClearBufferiv",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            buffer: u32,
            drawbuffer: i32,
            value: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let mut gl = gl_arc.borrow_mut();
                let memory =
                    MemoryHandle::new(store.data().lazy.get().unwrap().memory.clone(), &store);
                let converted_buffer = buffer;
                let converted_drawbuffer = drawbuffer;

                let len_value = (crate::compsize::glClearBufferiv_value_compsize(&mut gl, crate::ffi::GLEnumGroupBuffer::from_raw(buffer))) as usize;
                let mut vec_value: Vec<std::os::raw::c_int> = memory.read_vec::<i32>(value as u64, len_value as u64)?.iter().map(|v| *v as std::os::raw::c_int).collect::<Vec<_>>();
                let converted_value = vec_value.as_mut_ptr() as *const std::os::raw::c_int;

                let result = unsafe {
                    (gl.proc_addresses.glClearBufferiv)(
                        converted_buffer,
                        converted_drawbuffer,
                        converted_value,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glClearBufferuiv",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            buffer: u32,
            drawbuffer: i32,
            value: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let mut gl = gl_arc.borrow_mut();
                let memory =
                    MemoryHandle::new(store.data().lazy.get().unwrap().memory.clone(), &store);
                let converted_buffer = buffer;
                let converted_drawbuffer = drawbuffer;

                let len_value = (crate::compsize::glClearBufferuiv_value_compsize(&mut gl, crate::ffi::GLEnumGroupBuffer::from_raw(buffer))) as usize;
                let mut vec_value: Vec<std::os::raw::c_uint> = memory.read_vec::<u32>(value as u64, len_value as u64)?.iter().map(|v| *v as std::os::raw::c_uint).collect::<Vec<_>>();
                let converted_value = vec_value.as_mut_ptr() as *const std::os::raw::c_uint;

                let result = unsafe {
                    (gl.proc_addresses.glClearBufferuiv)(
                        converted_buffer,
                        converted_drawbuffer,
                        converted_value,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glClearColor",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            red: f32,
            green: f32,
            blue: f32,
            alpha: f32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let converted_red = red;
                let converted_green = green;
                let converted_blue = blue;
                let converted_alpha = alpha;
                let result = unsafe {
                    (gl.proc_addresses.glClearColor)(
                        converted_red,
                        converted_green,
                        converted_blue,
                        converted_alpha,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glClearDepthf",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            d: f32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let converted_d = d;
                let result = unsafe {
                    (gl.proc_addresses.glClearDepthf)(
                        converted_d,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glClearStencil",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            s: i32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let converted_s = s;
                let result = unsafe {
                    (gl.proc_addresses.glClearStencil)(
                        converted_s,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glClientWaitSync",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            sync: u32,
            flags: u32,
            timeout: u64,
        | -> Result<u32, wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let converted_sync = gl.resolve_opaque_sync_object(sync);
                let converted_flags = flags;
                let converted_timeout = timeout;
                let result = unsafe {
                    (gl.proc_addresses.glClientWaitSync)(
                        converted_sync,
                        converted_flags,
                        converted_timeout,
                    ) 
                };

                Ok(result.into())
            },
        ),
    );
    exports.insert(
    "glColorMask",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            red: u32,
            green: u32,
            blue: u32,
            alpha: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let converted_red = red as u8;
                let converted_green = green as u8;
                let converted_blue = blue as u8;
                let converted_alpha = alpha as u8;
                let result = unsafe {
                    (gl.proc_addresses.glColorMask)(
                        converted_red,
                        converted_green,
                        converted_blue,
                        converted_alpha,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glCompileShader",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            shader: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let converted_shader = shader;
                let result = unsafe {
                    (gl.proc_addresses.glCompileShader)(
                        converted_shader,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glCompressedTexImage2D",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            target: u32,
            level: i32,
            internalformat: u32,
            width: i32,
            height: i32,
            border: i32,
            imageSize: i32,
            data: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let memory =
                    MemoryHandle::new(store.data().lazy.get().unwrap().memory.clone(), &store);
                let converted_target = target;
                let converted_level = level;
                let converted_internalformat = internalformat;
                let converted_width = width;
                let converted_height = height;
                let converted_border = border;
                let converted_imageSize = imageSize;

                let len_data = (imageSize) as usize;
                let mut vec_data: Vec<u8> = memory.read_vec::<u8>(data as u64, len_data as u64)?;
                let converted_data = vec_data.as_mut_ptr() as *const ();

                let result = unsafe {
                    (gl.proc_addresses.glCompressedTexImage2D)(
                        converted_target,
                        converted_level,
                        converted_internalformat,
                        converted_width,
                        converted_height,
                        converted_border,
                        converted_imageSize,
                        converted_data,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glCompressedTexImage3D",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            target: u32,
            level: i32,
            internalformat: u32,
            width: i32,
            height: i32,
            depth: i32,
            border: i32,
            imageSize: i32,
            data: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let memory =
                    MemoryHandle::new(store.data().lazy.get().unwrap().memory.clone(), &store);
                let converted_target = target;
                let converted_level = level;
                let converted_internalformat = internalformat;
                let converted_width = width;
                let converted_height = height;
                let converted_depth = depth;
                let converted_border = border;
                let converted_imageSize = imageSize;

                let len_data = (imageSize) as usize;
                let mut vec_data: Vec<u8> = memory.read_vec::<u8>(data as u64, len_data as u64)?;
                let converted_data = vec_data.as_mut_ptr() as *const ();

                let result = unsafe {
                    (gl.proc_addresses.glCompressedTexImage3D)(
                        converted_target,
                        converted_level,
                        converted_internalformat,
                        converted_width,
                        converted_height,
                        converted_depth,
                        converted_border,
                        converted_imageSize,
                        converted_data,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glCompressedTexSubImage2D",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            target: u32,
            level: i32,
            xoffset: i32,
            yoffset: i32,
            width: i32,
            height: i32,
            format: u32,
            imageSize: i32,
            data: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let memory =
                    MemoryHandle::new(store.data().lazy.get().unwrap().memory.clone(), &store);
                let converted_target = target;
                let converted_level = level;
                let converted_xoffset = xoffset;
                let converted_yoffset = yoffset;
                let converted_width = width;
                let converted_height = height;
                let converted_format = format;
                let converted_imageSize = imageSize;

                let len_data = (imageSize) as usize;
                let mut vec_data: Vec<u8> = memory.read_vec::<u8>(data as u64, len_data as u64)?;
                let converted_data = vec_data.as_mut_ptr() as *const ();

                let result = unsafe {
                    (gl.proc_addresses.glCompressedTexSubImage2D)(
                        converted_target,
                        converted_level,
                        converted_xoffset,
                        converted_yoffset,
                        converted_width,
                        converted_height,
                        converted_format,
                        converted_imageSize,
                        converted_data,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glCompressedTexSubImage3D",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            target: u32,
            level: i32,
            xoffset: i32,
            yoffset: i32,
            zoffset: i32,
            width: i32,
            height: i32,
            depth: i32,
            format: u32,
            imageSize: i32,
            data: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let memory =
                    MemoryHandle::new(store.data().lazy.get().unwrap().memory.clone(), &store);
                let converted_target = target;
                let converted_level = level;
                let converted_xoffset = xoffset;
                let converted_yoffset = yoffset;
                let converted_zoffset = zoffset;
                let converted_width = width;
                let converted_height = height;
                let converted_depth = depth;
                let converted_format = format;
                let converted_imageSize = imageSize;

                let len_data = (imageSize) as usize;
                let mut vec_data: Vec<u8> = memory.read_vec::<u8>(data as u64, len_data as u64)?;
                let converted_data = vec_data.as_mut_ptr() as *const ();

                let result = unsafe {
                    (gl.proc_addresses.glCompressedTexSubImage3D)(
                        converted_target,
                        converted_level,
                        converted_xoffset,
                        converted_yoffset,
                        converted_zoffset,
                        converted_width,
                        converted_height,
                        converted_depth,
                        converted_format,
                        converted_imageSize,
                        converted_data,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glCopyBufferSubData",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            readTarget: u32,
            writeTarget: u32,
            readOffset: i32,
            writeOffset: i32,
            size: i32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let converted_readTarget = readTarget;
                let converted_writeTarget = writeTarget;
                let converted_readOffset = readOffset as isize;
                let converted_writeOffset = writeOffset as isize;
                let converted_size = size as isize;
                let result = unsafe {
                    (gl.proc_addresses.glCopyBufferSubData)(
                        converted_readTarget,
                        converted_writeTarget,
                        converted_readOffset,
                        converted_writeOffset,
                        converted_size,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glCopyTexImage2D",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            target: u32,
            level: i32,
            internalformat: u32,
            x: i32,
            y: i32,
            width: i32,
            height: i32,
            border: i32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let converted_target = target;
                let converted_level = level;
                let converted_internalformat = internalformat;
                let converted_x = x;
                let converted_y = y;
                let converted_width = width;
                let converted_height = height;
                let converted_border = border;
                let result = unsafe {
                    (gl.proc_addresses.glCopyTexImage2D)(
                        converted_target,
                        converted_level,
                        converted_internalformat,
                        converted_x,
                        converted_y,
                        converted_width,
                        converted_height,
                        converted_border,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glCopyTexSubImage2D",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            target: u32,
            level: i32,
            xoffset: i32,
            yoffset: i32,
            x: i32,
            y: i32,
            width: i32,
            height: i32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let converted_target = target;
                let converted_level = level;
                let converted_xoffset = xoffset;
                let converted_yoffset = yoffset;
                let converted_x = x;
                let converted_y = y;
                let converted_width = width;
                let converted_height = height;
                let result = unsafe {
                    (gl.proc_addresses.glCopyTexSubImage2D)(
                        converted_target,
                        converted_level,
                        converted_xoffset,
                        converted_yoffset,
                        converted_x,
                        converted_y,
                        converted_width,
                        converted_height,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glCopyTexSubImage3D",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            target: u32,
            level: i32,
            xoffset: i32,
            yoffset: i32,
            zoffset: i32,
            x: i32,
            y: i32,
            width: i32,
            height: i32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let converted_target = target;
                let converted_level = level;
                let converted_xoffset = xoffset;
                let converted_yoffset = yoffset;
                let converted_zoffset = zoffset;
                let converted_x = x;
                let converted_y = y;
                let converted_width = width;
                let converted_height = height;
                let result = unsafe {
                    (gl.proc_addresses.glCopyTexSubImage3D)(
                        converted_target,
                        converted_level,
                        converted_xoffset,
                        converted_yoffset,
                        converted_zoffset,
                        converted_x,
                        converted_y,
                        converted_width,
                        converted_height,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glCreateProgram",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
        | -> Result<u32, wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();

                let result = unsafe {
                    (gl.proc_addresses.glCreateProgram)(
                    ) 
                };

                Ok(result.into())
            },
        ),
    );
    exports.insert(
    "glCreateShader",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            _type: u32,
        | -> Result<u32, wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let converted__type = _type;
                let result = unsafe {
                    (gl.proc_addresses.glCreateShader)(
                        converted__type,
                    ) 
                };

                Ok(result.into())
            },
        ),
    );
    exports.insert(
    "glCullFace",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            mode: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let converted_mode = mode;
                let result = unsafe {
                    (gl.proc_addresses.glCullFace)(
                        converted_mode,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glDeleteBuffers",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            n: i32,
            buffers: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let memory =
                    MemoryHandle::new(store.data().lazy.get().unwrap().memory.clone(), &store);
                let converted_n = n;

                let len_buffers = (n) as usize;
                let mut vec_buffers: Vec<std::os::raw::c_uint> = memory.read_vec::<u32>(buffers as u64, len_buffers as u64)?.iter().map(|v| *v as std::os::raw::c_uint).collect::<Vec<_>>();
                let converted_buffers = vec_buffers.as_mut_ptr() as *const std::os::raw::c_uint;

                let result = unsafe {
                    (gl.proc_addresses.glDeleteBuffers)(
                        converted_n,
                        converted_buffers,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glDeleteFramebuffers",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            n: i32,
            framebuffers: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let memory =
                    MemoryHandle::new(store.data().lazy.get().unwrap().memory.clone(), &store);
                let converted_n = n;

                let len_framebuffers = (n) as usize;
                let mut vec_framebuffers: Vec<std::os::raw::c_uint> = memory.read_vec::<u32>(framebuffers as u64, len_framebuffers as u64)?.iter().map(|v| *v as std::os::raw::c_uint).collect::<Vec<_>>();
                let converted_framebuffers = vec_framebuffers.as_mut_ptr() as *const std::os::raw::c_uint;

                let result = unsafe {
                    (gl.proc_addresses.glDeleteFramebuffers)(
                        converted_n,
                        converted_framebuffers,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glDeleteProgram",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            program: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let converted_program = program;
                let result = unsafe {
                    (gl.proc_addresses.glDeleteProgram)(
                        converted_program,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glDeleteQueries",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            n: i32,
            ids: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let memory =
                    MemoryHandle::new(store.data().lazy.get().unwrap().memory.clone(), &store);
                let converted_n = n;

                let len_ids = (n) as usize;
                let mut vec_ids: Vec<std::os::raw::c_uint> = memory.read_vec::<u32>(ids as u64, len_ids as u64)?.iter().map(|v| *v as std::os::raw::c_uint).collect::<Vec<_>>();
                let converted_ids = vec_ids.as_mut_ptr() as *const std::os::raw::c_uint;

                let result = unsafe {
                    (gl.proc_addresses.glDeleteQueries)(
                        converted_n,
                        converted_ids,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glDeleteRenderbuffers",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            n: i32,
            renderbuffers: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let memory =
                    MemoryHandle::new(store.data().lazy.get().unwrap().memory.clone(), &store);
                let converted_n = n;

                let len_renderbuffers = (n) as usize;
                let mut vec_renderbuffers: Vec<std::os::raw::c_uint> = memory.read_vec::<u32>(renderbuffers as u64, len_renderbuffers as u64)?.iter().map(|v| *v as std::os::raw::c_uint).collect::<Vec<_>>();
                let converted_renderbuffers = vec_renderbuffers.as_mut_ptr() as *const std::os::raw::c_uint;

                let result = unsafe {
                    (gl.proc_addresses.glDeleteRenderbuffers)(
                        converted_n,
                        converted_renderbuffers,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glDeleteSamplers",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            count: i32,
            samplers: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let memory =
                    MemoryHandle::new(store.data().lazy.get().unwrap().memory.clone(), &store);
                let converted_count = count;

                let len_samplers = (count) as usize;
                let mut vec_samplers: Vec<std::os::raw::c_uint> = memory.read_vec::<u32>(samplers as u64, len_samplers as u64)?.iter().map(|v| *v as std::os::raw::c_uint).collect::<Vec<_>>();
                let converted_samplers = vec_samplers.as_mut_ptr() as *const std::os::raw::c_uint;

                let result = unsafe {
                    (gl.proc_addresses.glDeleteSamplers)(
                        converted_count,
                        converted_samplers,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glDeleteShader",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            shader: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let converted_shader = shader;
                let result = unsafe {
                    (gl.proc_addresses.glDeleteShader)(
                        converted_shader,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glDeleteSync",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            sync: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let converted_sync = gl.resolve_opaque_sync_object(sync);
                let result = unsafe {
                    (gl.proc_addresses.glDeleteSync)(
                        converted_sync,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glDeleteTextures",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            n: i32,
            textures: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let memory =
                    MemoryHandle::new(store.data().lazy.get().unwrap().memory.clone(), &store);
                let converted_n = n;

                let len_textures = (n) as usize;
                let mut vec_textures: Vec<std::os::raw::c_uint> = memory.read_vec::<u32>(textures as u64, len_textures as u64)?.iter().map(|v| *v as std::os::raw::c_uint).collect::<Vec<_>>();
                let converted_textures = vec_textures.as_mut_ptr() as *const std::os::raw::c_uint;

                let result = unsafe {
                    (gl.proc_addresses.glDeleteTextures)(
                        converted_n,
                        converted_textures,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glDeleteTransformFeedbacks",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            n: i32,
            ids: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let memory =
                    MemoryHandle::new(store.data().lazy.get().unwrap().memory.clone(), &store);
                let converted_n = n;

                let len_ids = (n) as usize;
                let mut vec_ids: Vec<std::os::raw::c_uint> = memory.read_vec::<u32>(ids as u64, len_ids as u64)?.iter().map(|v| *v as std::os::raw::c_uint).collect::<Vec<_>>();
                let converted_ids = vec_ids.as_mut_ptr() as *const std::os::raw::c_uint;

                let result = unsafe {
                    (gl.proc_addresses.glDeleteTransformFeedbacks)(
                        converted_n,
                        converted_ids,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glDeleteVertexArrays",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            n: i32,
            arrays: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let memory =
                    MemoryHandle::new(store.data().lazy.get().unwrap().memory.clone(), &store);
                let converted_n = n;

                let len_arrays = (n) as usize;
                let mut vec_arrays: Vec<std::os::raw::c_uint> = memory.read_vec::<u32>(arrays as u64, len_arrays as u64)?.iter().map(|v| *v as std::os::raw::c_uint).collect::<Vec<_>>();
                let converted_arrays = vec_arrays.as_mut_ptr() as *const std::os::raw::c_uint;

                let result = unsafe {
                    (gl.proc_addresses.glDeleteVertexArrays)(
                        converted_n,
                        converted_arrays,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glDeleteVertexArraysOES",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            n: i32,
            arrays: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let memory =
                    MemoryHandle::new(store.data().lazy.get().unwrap().memory.clone(), &store);
                let converted_n = n;

                let len_arrays = (n) as usize;
                let mut vec_arrays: Vec<std::os::raw::c_uint> = memory.read_vec::<u32>(arrays as u64, len_arrays as u64)?.iter().map(|v| *v as std::os::raw::c_uint).collect::<Vec<_>>();
                let converted_arrays = vec_arrays.as_mut_ptr() as *const std::os::raw::c_uint;

                let result = unsafe {
                    (gl.proc_addresses.glDeleteVertexArraysOES)(
                        converted_n,
                        converted_arrays,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glDepthFunc",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            func: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let converted_func = func;
                let result = unsafe {
                    (gl.proc_addresses.glDepthFunc)(
                        converted_func,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glDepthMask",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            flag: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let converted_flag = flag as u8;
                let result = unsafe {
                    (gl.proc_addresses.glDepthMask)(
                        converted_flag,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glDepthRangef",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            n: f32,
            f: f32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let converted_n = n;
                let converted_f = f;
                let result = unsafe {
                    (gl.proc_addresses.glDepthRangef)(
                        converted_n,
                        converted_f,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glDetachShader",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            program: u32,
            shader: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let converted_program = program;
                let converted_shader = shader;
                let result = unsafe {
                    (gl.proc_addresses.glDetachShader)(
                        converted_program,
                        converted_shader,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glDisable",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            cap: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let converted_cap = cap;
                let result = unsafe {
                    (gl.proc_addresses.glDisable)(
                        converted_cap,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glDisableVertexAttribArray",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            index: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let converted_index = index;
                let result = unsafe {
                    (gl.proc_addresses.glDisableVertexAttribArray)(
                        converted_index,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glDrawArrays",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            mode: u32,
            first: i32,
            count: i32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let converted_mode = mode;
                let converted_first = first;
                let converted_count = count;
                let result = unsafe {
                    (gl.proc_addresses.glDrawArrays)(
                        converted_mode,
                        converted_first,
                        converted_count,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glDrawArraysInstanced",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            mode: u32,
            first: i32,
            count: i32,
            instancecount: i32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let converted_mode = mode;
                let converted_first = first;
                let converted_count = count;
                let converted_instancecount = instancecount;
                let result = unsafe {
                    (gl.proc_addresses.glDrawArraysInstanced)(
                        converted_mode,
                        converted_first,
                        converted_count,
                        converted_instancecount,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glDrawBuffers",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            n: i32,
            bufs: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let memory =
                    MemoryHandle::new(store.data().lazy.get().unwrap().memory.clone(), &store);
                let converted_n = n;

                let len_bufs = (n) as usize;
                let mut vec_bufs: Vec<std::os::raw::c_uint> = memory.read_vec::<u32>(bufs as u64, len_bufs as u64)?.iter().map(|v| *v as std::os::raw::c_uint).collect::<Vec<_>>();
                let converted_bufs = vec_bufs.as_mut_ptr() as *const std::os::raw::c_uint;

                let result = unsafe {
                    (gl.proc_addresses.glDrawBuffers)(
                        converted_n,
                        converted_bufs,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glEnable",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            cap: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let converted_cap = cap;
                let result = unsafe {
                    (gl.proc_addresses.glEnable)(
                        converted_cap,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glEnableVertexAttribArray",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            index: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let converted_index = index;
                let result = unsafe {
                    (gl.proc_addresses.glEnableVertexAttribArray)(
                        converted_index,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glEndQuery",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            target: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let converted_target = target;
                let result = unsafe {
                    (gl.proc_addresses.glEndQuery)(
                        converted_target,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glEndTransformFeedback",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();

                let result = unsafe {
                    (gl.proc_addresses.glEndTransformFeedback)(
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glFenceSync",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            condition: u32,
            flags: u32,
        | -> Result<u32, wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let converted_condition = condition;
                let converted_flags = flags;
                let result = unsafe {
                    (gl.proc_addresses.glFenceSync)(
                        converted_condition,
                        converted_flags,
                    ) 
                };

                Ok(gl.register_opaque_sync_object(result))
            },
        ),
    );
    exports.insert(
    "glFinish",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();

                let result = unsafe {
                    (gl.proc_addresses.glFinish)(
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glFlush",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();

                let result = unsafe {
                    (gl.proc_addresses.glFlush)(
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glFlushMappedBufferRange",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            target: u32,
            offset: i32,
            length: i32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let converted_target = target;
                let converted_offset = offset as isize;
                let converted_length = length as isize;
                let result = unsafe {
                    (gl.proc_addresses.glFlushMappedBufferRange)(
                        converted_target,
                        converted_offset,
                        converted_length,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glFramebufferRenderbuffer",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            target: u32,
            attachment: u32,
            renderbuffertarget: u32,
            renderbuffer: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let converted_target = target;
                let converted_attachment = attachment;
                let converted_renderbuffertarget = renderbuffertarget;
                let converted_renderbuffer = renderbuffer;
                let result = unsafe {
                    (gl.proc_addresses.glFramebufferRenderbuffer)(
                        converted_target,
                        converted_attachment,
                        converted_renderbuffertarget,
                        converted_renderbuffer,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glFramebufferTexture2D",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            target: u32,
            attachment: u32,
            textarget: u32,
            texture: u32,
            level: i32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let converted_target = target;
                let converted_attachment = attachment;
                let converted_textarget = textarget;
                let converted_texture = texture;
                let converted_level = level;
                let result = unsafe {
                    (gl.proc_addresses.glFramebufferTexture2D)(
                        converted_target,
                        converted_attachment,
                        converted_textarget,
                        converted_texture,
                        converted_level,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glFramebufferTextureLayer",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            target: u32,
            attachment: u32,
            texture: u32,
            level: i32,
            layer: i32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let converted_target = target;
                let converted_attachment = attachment;
                let converted_texture = texture;
                let converted_level = level;
                let converted_layer = layer;
                let result = unsafe {
                    (gl.proc_addresses.glFramebufferTextureLayer)(
                        converted_target,
                        converted_attachment,
                        converted_texture,
                        converted_level,
                        converted_layer,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glFrontFace",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            mode: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let converted_mode = mode;
                let result = unsafe {
                    (gl.proc_addresses.glFrontFace)(
                        converted_mode,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glGenBuffers",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            n: i32,
            buffers: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let memory =
                    MemoryHandle::new(store.data().lazy.get().unwrap().memory.clone(), &store);
                let converted_n = n;

                let len_buffers = (n) as usize;
                let mut vec_buffers: Vec<std::os::raw::c_uint> = memory.read_vec::<u32>(buffers as u64, len_buffers as u64)?.iter().map(|v| *v as std::os::raw::c_uint).collect::<Vec<_>>();
                let converted_buffers = vec_buffers.as_mut_ptr() as *mut std::os::raw::c_uint;

                let result = unsafe {
                    (gl.proc_addresses.glGenBuffers)(
                        converted_n,
                        converted_buffers,
                    ) 
                };

                memory.write_slice::<u32>(buffers as u64, &vec_buffers.iter().map(|v| *v as _).collect::<Vec<_>>())?;

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glGenFramebuffers",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            n: i32,
            framebuffers: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let memory =
                    MemoryHandle::new(store.data().lazy.get().unwrap().memory.clone(), &store);
                let converted_n = n;

                let len_framebuffers = (n) as usize;
                let mut vec_framebuffers: Vec<std::os::raw::c_uint> = memory.read_vec::<u32>(framebuffers as u64, len_framebuffers as u64)?.iter().map(|v| *v as std::os::raw::c_uint).collect::<Vec<_>>();
                let converted_framebuffers = vec_framebuffers.as_mut_ptr() as *mut std::os::raw::c_uint;

                let result = unsafe {
                    (gl.proc_addresses.glGenFramebuffers)(
                        converted_n,
                        converted_framebuffers,
                    ) 
                };

                memory.write_slice::<u32>(framebuffers as u64, &vec_framebuffers.iter().map(|v| *v as _).collect::<Vec<_>>())?;

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glGenQueries",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            n: i32,
            ids: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let memory =
                    MemoryHandle::new(store.data().lazy.get().unwrap().memory.clone(), &store);
                let converted_n = n;

                let len_ids = (n) as usize;
                let mut vec_ids: Vec<std::os::raw::c_uint> = memory.read_vec::<u32>(ids as u64, len_ids as u64)?.iter().map(|v| *v as std::os::raw::c_uint).collect::<Vec<_>>();
                let converted_ids = vec_ids.as_mut_ptr() as *mut std::os::raw::c_uint;

                let result = unsafe {
                    (gl.proc_addresses.glGenQueries)(
                        converted_n,
                        converted_ids,
                    ) 
                };

                memory.write_slice::<u32>(ids as u64, &vec_ids.iter().map(|v| *v as _).collect::<Vec<_>>())?;

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glGenRenderbuffers",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            n: i32,
            renderbuffers: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let memory =
                    MemoryHandle::new(store.data().lazy.get().unwrap().memory.clone(), &store);
                let converted_n = n;

                let len_renderbuffers = (n) as usize;
                let mut vec_renderbuffers: Vec<std::os::raw::c_uint> = memory.read_vec::<u32>(renderbuffers as u64, len_renderbuffers as u64)?.iter().map(|v| *v as std::os::raw::c_uint).collect::<Vec<_>>();
                let converted_renderbuffers = vec_renderbuffers.as_mut_ptr() as *mut std::os::raw::c_uint;

                let result = unsafe {
                    (gl.proc_addresses.glGenRenderbuffers)(
                        converted_n,
                        converted_renderbuffers,
                    ) 
                };

                memory.write_slice::<u32>(renderbuffers as u64, &vec_renderbuffers.iter().map(|v| *v as _).collect::<Vec<_>>())?;

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glGenSamplers",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            count: i32,
            samplers: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let memory =
                    MemoryHandle::new(store.data().lazy.get().unwrap().memory.clone(), &store);
                let converted_count = count;

                let len_samplers = (count) as usize;
                let mut vec_samplers: Vec<std::os::raw::c_uint> = memory.read_vec::<u32>(samplers as u64, len_samplers as u64)?.iter().map(|v| *v as std::os::raw::c_uint).collect::<Vec<_>>();
                let converted_samplers = vec_samplers.as_mut_ptr() as *mut std::os::raw::c_uint;

                let result = unsafe {
                    (gl.proc_addresses.glGenSamplers)(
                        converted_count,
                        converted_samplers,
                    ) 
                };

                memory.write_slice::<u32>(samplers as u64, &vec_samplers.iter().map(|v| *v as _).collect::<Vec<_>>())?;

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glGenTextures",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            n: i32,
            textures: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let memory =
                    MemoryHandle::new(store.data().lazy.get().unwrap().memory.clone(), &store);
                let converted_n = n;

                let len_textures = (n) as usize;
                let mut vec_textures: Vec<std::os::raw::c_uint> = memory.read_vec::<u32>(textures as u64, len_textures as u64)?.iter().map(|v| *v as std::os::raw::c_uint).collect::<Vec<_>>();
                let converted_textures = vec_textures.as_mut_ptr() as *mut std::os::raw::c_uint;

                let result = unsafe {
                    (gl.proc_addresses.glGenTextures)(
                        converted_n,
                        converted_textures,
                    ) 
                };

                memory.write_slice::<u32>(textures as u64, &vec_textures.iter().map(|v| *v as _).collect::<Vec<_>>())?;

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glGenTransformFeedbacks",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            n: i32,
            ids: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let memory =
                    MemoryHandle::new(store.data().lazy.get().unwrap().memory.clone(), &store);
                let converted_n = n;

                let len_ids = (n) as usize;
                let mut vec_ids: Vec<std::os::raw::c_uint> = memory.read_vec::<u32>(ids as u64, len_ids as u64)?.iter().map(|v| *v as std::os::raw::c_uint).collect::<Vec<_>>();
                let converted_ids = vec_ids.as_mut_ptr() as *mut std::os::raw::c_uint;

                let result = unsafe {
                    (gl.proc_addresses.glGenTransformFeedbacks)(
                        converted_n,
                        converted_ids,
                    ) 
                };

                memory.write_slice::<u32>(ids as u64, &vec_ids.iter().map(|v| *v as _).collect::<Vec<_>>())?;

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glGenVertexArrays",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            n: i32,
            arrays: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let memory =
                    MemoryHandle::new(store.data().lazy.get().unwrap().memory.clone(), &store);
                let converted_n = n;

                let len_arrays = (n) as usize;
                let mut vec_arrays: Vec<std::os::raw::c_uint> = memory.read_vec::<u32>(arrays as u64, len_arrays as u64)?.iter().map(|v| *v as std::os::raw::c_uint).collect::<Vec<_>>();
                let converted_arrays = vec_arrays.as_mut_ptr() as *mut std::os::raw::c_uint;

                let result = unsafe {
                    (gl.proc_addresses.glGenVertexArrays)(
                        converted_n,
                        converted_arrays,
                    ) 
                };

                memory.write_slice::<u32>(arrays as u64, &vec_arrays.iter().map(|v| *v as _).collect::<Vec<_>>())?;

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glGenVertexArraysOES",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            n: i32,
            arrays: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let memory =
                    MemoryHandle::new(store.data().lazy.get().unwrap().memory.clone(), &store);
                let converted_n = n;

                let len_arrays = (n) as usize;
                let mut vec_arrays: Vec<std::os::raw::c_uint> = memory.read_vec::<u32>(arrays as u64, len_arrays as u64)?.iter().map(|v| *v as std::os::raw::c_uint).collect::<Vec<_>>();
                let converted_arrays = vec_arrays.as_mut_ptr() as *mut std::os::raw::c_uint;

                let result = unsafe {
                    (gl.proc_addresses.glGenVertexArraysOES)(
                        converted_n,
                        converted_arrays,
                    ) 
                };

                memory.write_slice::<u32>(arrays as u64, &vec_arrays.iter().map(|v| *v as _).collect::<Vec<_>>())?;

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glGenerateMipmap",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            target: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let converted_target = target;
                let result = unsafe {
                    (gl.proc_addresses.glGenerateMipmap)(
                        converted_target,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glGetActiveAttrib",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            program: u32,
            index: u32,
            bufSize: i32,
            length: u32,
            size: u32,
            _type: u32,
            name: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let memory =
                    MemoryHandle::new(store.data().lazy.get().unwrap().memory.clone(), &store);
                let converted_program = program;
                let converted_index = index;
                let converted_bufSize = bufSize;

                let len_length = (1) as usize;
                let mut vec_length: Vec<std::os::raw::c_int> = memory.read_vec::<i32>(length as u64, len_length as u64)?.iter().map(|v| *v as std::os::raw::c_int).collect::<Vec<_>>();
                let converted_length = vec_length.as_mut_ptr() as *mut std::os::raw::c_int;


                let len_size = (1) as usize;
                let mut vec_size: Vec<std::os::raw::c_int> = memory.read_vec::<i32>(size as u64, len_size as u64)?.iter().map(|v| *v as std::os::raw::c_int).collect::<Vec<_>>();
                let converted_size = vec_size.as_mut_ptr() as *mut std::os::raw::c_int;


                let len__type = (1) as usize;
                let mut vec__type: Vec<std::os::raw::c_uint> = memory.read_vec::<u32>(_type as u64, len__type as u64)?.iter().map(|v| *v as std::os::raw::c_uint).collect::<Vec<_>>();
                let converted__type = vec__type.as_mut_ptr() as *mut std::os::raw::c_uint;


                let len_name = (bufSize) as usize;
                let mut vec_name: Vec<i8> = memory.read_vec::<i8>(name as u64, len_name as u64)?;
                let converted_name = vec_name.as_mut_ptr() as *mut i8;

                let result = unsafe {
                    (gl.proc_addresses.glGetActiveAttrib)(
                        converted_program,
                        converted_index,
                        converted_bufSize,
                        converted_length,
                        converted_size,
                        converted__type,
                        converted_name,
                    ) 
                };

                memory.write_slice::<i32>(length as u64, &vec_length.iter().map(|v| *v as _).collect::<Vec<_>>())?;


                memory.write_slice::<i32>(size as u64, &vec_size.iter().map(|v| *v as _).collect::<Vec<_>>())?;


                memory.write_slice::<u32>(_type as u64, &vec__type.iter().map(|v| *v as _).collect::<Vec<_>>())?;


                memory.write_slice::<i8>(name as u64, &vec_name)?;

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glGetActiveUniform",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            program: u32,
            index: u32,
            bufSize: i32,
            length: u32,
            size: u32,
            _type: u32,
            name: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let memory =
                    MemoryHandle::new(store.data().lazy.get().unwrap().memory.clone(), &store);
                let converted_program = program;
                let converted_index = index;
                let converted_bufSize = bufSize;

                let len_length = (1) as usize;
                let mut vec_length: Vec<std::os::raw::c_int> = memory.read_vec::<i32>(length as u64, len_length as u64)?.iter().map(|v| *v as std::os::raw::c_int).collect::<Vec<_>>();
                let converted_length = vec_length.as_mut_ptr() as *mut std::os::raw::c_int;


                let len_size = (1) as usize;
                let mut vec_size: Vec<std::os::raw::c_int> = memory.read_vec::<i32>(size as u64, len_size as u64)?.iter().map(|v| *v as std::os::raw::c_int).collect::<Vec<_>>();
                let converted_size = vec_size.as_mut_ptr() as *mut std::os::raw::c_int;


                let len__type = (1) as usize;
                let mut vec__type: Vec<std::os::raw::c_uint> = memory.read_vec::<u32>(_type as u64, len__type as u64)?.iter().map(|v| *v as std::os::raw::c_uint).collect::<Vec<_>>();
                let converted__type = vec__type.as_mut_ptr() as *mut std::os::raw::c_uint;


                let len_name = (bufSize) as usize;
                let mut vec_name: Vec<i8> = memory.read_vec::<i8>(name as u64, len_name as u64)?;
                let converted_name = vec_name.as_mut_ptr() as *mut i8;

                let result = unsafe {
                    (gl.proc_addresses.glGetActiveUniform)(
                        converted_program,
                        converted_index,
                        converted_bufSize,
                        converted_length,
                        converted_size,
                        converted__type,
                        converted_name,
                    ) 
                };

                memory.write_slice::<i32>(length as u64, &vec_length.iter().map(|v| *v as _).collect::<Vec<_>>())?;


                memory.write_slice::<i32>(size as u64, &vec_size.iter().map(|v| *v as _).collect::<Vec<_>>())?;


                memory.write_slice::<u32>(_type as u64, &vec__type.iter().map(|v| *v as _).collect::<Vec<_>>())?;


                memory.write_slice::<i8>(name as u64, &vec_name)?;

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glGetActiveUniformBlockName",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            program: u32,
            uniformBlockIndex: u32,
            bufSize: i32,
            length: u32,
            uniformBlockName: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let memory =
                    MemoryHandle::new(store.data().lazy.get().unwrap().memory.clone(), &store);
                let converted_program = program;
                let converted_uniformBlockIndex = uniformBlockIndex;
                let converted_bufSize = bufSize;

                let len_length = (1) as usize;
                let mut vec_length: Vec<std::os::raw::c_int> = memory.read_vec::<i32>(length as u64, len_length as u64)?.iter().map(|v| *v as std::os::raw::c_int).collect::<Vec<_>>();
                let converted_length = vec_length.as_mut_ptr() as *mut std::os::raw::c_int;


                let len_uniformBlockName = (bufSize) as usize;
                let mut vec_uniformBlockName: Vec<i8> = memory.read_vec::<i8>(uniformBlockName as u64, len_uniformBlockName as u64)?;
                let converted_uniformBlockName = vec_uniformBlockName.as_mut_ptr() as *mut i8;

                let result = unsafe {
                    (gl.proc_addresses.glGetActiveUniformBlockName)(
                        converted_program,
                        converted_uniformBlockIndex,
                        converted_bufSize,
                        converted_length,
                        converted_uniformBlockName,
                    ) 
                };

                memory.write_slice::<i32>(length as u64, &vec_length.iter().map(|v| *v as _).collect::<Vec<_>>())?;


                memory.write_slice::<i8>(uniformBlockName as u64, &vec_uniformBlockName)?;

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glGetActiveUniformBlockiv",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            program: u32,
            uniformBlockIndex: u32,
            pname: u32,
            params: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let mut gl = gl_arc.borrow_mut();
                let memory =
                    MemoryHandle::new(store.data().lazy.get().unwrap().memory.clone(), &store);
                let converted_program = program;
                let converted_uniformBlockIndex = uniformBlockIndex;
                let converted_pname = pname;

                let len_params = (crate::compsize::glGetActiveUniformBlockiv_params_compsize(&mut gl, program,uniformBlockIndex,crate::ffi::GLEnumGroupUniformBlockPName::from_raw(pname))) as usize;
                let mut vec_params: Vec<std::os::raw::c_int> = memory.read_vec::<i32>(params as u64, len_params as u64)?.iter().map(|v| *v as std::os::raw::c_int).collect::<Vec<_>>();
                let converted_params = vec_params.as_mut_ptr() as *mut std::os::raw::c_int;

                let result = unsafe {
                    (gl.proc_addresses.glGetActiveUniformBlockiv)(
                        converted_program,
                        converted_uniformBlockIndex,
                        converted_pname,
                        converted_params,
                    ) 
                };

                memory.write_slice::<i32>(params as u64, &vec_params.iter().map(|v| *v as _).collect::<Vec<_>>())?;

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glGetActiveUniformsiv",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            program: u32,
            uniformCount: i32,
            uniformIndices: u32,
            pname: u32,
            params: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let mut gl = gl_arc.borrow_mut();
                let memory =
                    MemoryHandle::new(store.data().lazy.get().unwrap().memory.clone(), &store);
                let converted_program = program;
                let converted_uniformCount = uniformCount;

                let len_uniformIndices = (uniformCount) as usize;
                let mut vec_uniformIndices: Vec<std::os::raw::c_uint> = memory.read_vec::<u32>(uniformIndices as u64, len_uniformIndices as u64)?.iter().map(|v| *v as std::os::raw::c_uint).collect::<Vec<_>>();
                let converted_uniformIndices = vec_uniformIndices.as_mut_ptr() as *const std::os::raw::c_uint;

                let converted_pname = pname;

                let len_params = (crate::compsize::glGetActiveUniformsiv_params_compsize(&mut gl, uniformCount,crate::ffi::GLEnumGroupUniformPName::from_raw(pname))) as usize;
                let mut vec_params: Vec<std::os::raw::c_int> = memory.read_vec::<i32>(params as u64, len_params as u64)?.iter().map(|v| *v as std::os::raw::c_int).collect::<Vec<_>>();
                let converted_params = vec_params.as_mut_ptr() as *mut std::os::raw::c_int;

                let result = unsafe {
                    (gl.proc_addresses.glGetActiveUniformsiv)(
                        converted_program,
                        converted_uniformCount,
                        converted_uniformIndices,
                        converted_pname,
                        converted_params,
                    ) 
                };

                memory.write_slice::<i32>(params as u64, &vec_params.iter().map(|v| *v as _).collect::<Vec<_>>())?;

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glGetAttachedShaders",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            program: u32,
            maxCount: i32,
            count: u32,
            shaders: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let memory =
                    MemoryHandle::new(store.data().lazy.get().unwrap().memory.clone(), &store);
                let converted_program = program;
                let converted_maxCount = maxCount;

                let len_count = (1) as usize;
                let mut vec_count: Vec<std::os::raw::c_int> = memory.read_vec::<i32>(count as u64, len_count as u64)?.iter().map(|v| *v as std::os::raw::c_int).collect::<Vec<_>>();
                let converted_count = vec_count.as_mut_ptr() as *mut std::os::raw::c_int;


                let len_shaders = (maxCount) as usize;
                let mut vec_shaders: Vec<std::os::raw::c_uint> = memory.read_vec::<u32>(shaders as u64, len_shaders as u64)?.iter().map(|v| *v as std::os::raw::c_uint).collect::<Vec<_>>();
                let converted_shaders = vec_shaders.as_mut_ptr() as *mut std::os::raw::c_uint;

                let result = unsafe {
                    (gl.proc_addresses.glGetAttachedShaders)(
                        converted_program,
                        converted_maxCount,
                        converted_count,
                        converted_shaders,
                    ) 
                };

                memory.write_slice::<i32>(count as u64, &vec_count.iter().map(|v| *v as _).collect::<Vec<_>>())?;


                memory.write_slice::<u32>(shaders as u64, &vec_shaders.iter().map(|v| *v as _).collect::<Vec<_>>())?;

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glGetAttribLocation",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            program: u32,
            name: u32,
        | -> Result<i32, wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let memory =
                    MemoryHandle::new(store.data().lazy.get().unwrap().memory.clone(), &store);
                let converted_program = program;

                let len_name = (crate::utils::guest_strlen(&memory, name as u64) + 1) as usize;
                let mut vec_name: Vec<i8> = memory.read_vec::<i8>(name as u64, len_name as u64)?;
                let converted_name = vec_name.as_mut_ptr() as *const i8;

                let result = unsafe {
                    (gl.proc_addresses.glGetAttribLocation)(
                        converted_program,
                        converted_name,
                    ) 
                };

                Ok(result.into())
            },
        ),
    );
    exports.insert(
    "glGetBooleanv",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            pname: u32,
            data: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let mut gl = gl_arc.borrow_mut();
                let memory =
                    MemoryHandle::new(store.data().lazy.get().unwrap().memory.clone(), &store);
                let converted_pname = pname;

                let len_data = (crate::compsize::glGetBooleanv_data_compsize(&mut gl, crate::ffi::GLEnumGroupGetPName::from_raw(pname))) as usize;
                let mut vec_data: Vec<u8> = memory.read_vec::<u8>(data as u64, len_data as u64)?;
                let converted_data = vec_data.as_mut_ptr() as *mut u8;

                let result = unsafe {
                    (gl.proc_addresses.glGetBooleanv)(
                        converted_pname,
                        converted_data,
                    ) 
                };

                memory.write_slice::<u8>(data as u64, &vec_data)?;

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glGetBufferParameteri64v",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            target: u32,
            pname: u32,
            params: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let mut gl = gl_arc.borrow_mut();
                let memory =
                    MemoryHandle::new(store.data().lazy.get().unwrap().memory.clone(), &store);
                let converted_target = target;
                let converted_pname = pname;

                let len_params = (crate::compsize::glGetBufferParameteri64v_params_compsize(&mut gl, crate::ffi::GLEnumGroupBufferPNameARB::from_raw(pname))) as usize;
                let mut vec_params: Vec<i64> = memory.read_vec::<i64>(params as u64, len_params as u64)?;
                let converted_params = vec_params.as_mut_ptr() as *mut i64;

                let result = unsafe {
                    (gl.proc_addresses.glGetBufferParameteri64v)(
                        converted_target,
                        converted_pname,
                        converted_params,
                    ) 
                };

                memory.write_slice::<i64>(params as u64, &vec_params)?;

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glGetBufferParameteriv",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            target: u32,
            pname: u32,
            params: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let mut gl = gl_arc.borrow_mut();
                let memory =
                    MemoryHandle::new(store.data().lazy.get().unwrap().memory.clone(), &store);
                let converted_target = target;
                let converted_pname = pname;

                let len_params = (crate::compsize::glGetBufferParameteriv_params_compsize(&mut gl, crate::ffi::GLEnumGroupBufferPNameARB::from_raw(pname))) as usize;
                let mut vec_params: Vec<std::os::raw::c_int> = memory.read_vec::<i32>(params as u64, len_params as u64)?.iter().map(|v| *v as std::os::raw::c_int).collect::<Vec<_>>();
                let converted_params = vec_params.as_mut_ptr() as *mut std::os::raw::c_int;

                let result = unsafe {
                    (gl.proc_addresses.glGetBufferParameteriv)(
                        converted_target,
                        converted_pname,
                        converted_params,
                    ) 
                };

                memory.write_slice::<i32>(params as u64, &vec_params.iter().map(|v| *v as _).collect::<Vec<_>>())?;

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glGetBufferPointerv",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            target: u32,
            pname: u32,
            params: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let memory =
                    MemoryHandle::new(store.data().lazy.get().unwrap().memory.clone(), &store);
                let converted_target = target;
                let converted_pname = pname;

                let len_params = (1) as usize;
                let mut vec_params: Vec<*mut ()> = memory.read_vec::<u32>(params as u64, len_params as u64)?.iter().map(|v| *v as *mut ()).collect::<Vec<_>>();
                let converted_params = vec_params.as_mut_ptr() as *mut *mut ();

                let result = unsafe {
                    (gl.proc_addresses.glGetBufferPointerv)(
                        converted_target,
                        converted_pname,
                        converted_params,
                    ) 
                };

                memory.write_slice::<u32>(params as u64, &vec_params.iter().map(|v| *v as _).collect::<Vec<_>>())?;

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glGetError",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
        | -> Result<u32, wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();

                let result = unsafe {
                    (gl.proc_addresses.glGetError)(
                    ) 
                };

                Ok(result.into())
            },
        ),
    );
    exports.insert(
    "glGetFloatv",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            pname: u32,
            data: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let mut gl = gl_arc.borrow_mut();
                let memory =
                    MemoryHandle::new(store.data().lazy.get().unwrap().memory.clone(), &store);
                let converted_pname = pname;

                let len_data = (crate::compsize::glGetFloatv_data_compsize(&mut gl, crate::ffi::GLEnumGroupGetPName::from_raw(pname))) as usize;
                let mut vec_data: Vec<f32> = memory.read_vec::<f32>(data as u64, len_data as u64)?;
                let converted_data = vec_data.as_mut_ptr() as *mut f32;

                let result = unsafe {
                    (gl.proc_addresses.glGetFloatv)(
                        converted_pname,
                        converted_data,
                    ) 
                };

                memory.write_slice::<f32>(data as u64, &vec_data)?;

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glGetFragDataLocation",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            program: u32,
            name: u32,
        | -> Result<i32, wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let memory =
                    MemoryHandle::new(store.data().lazy.get().unwrap().memory.clone(), &store);
                let converted_program = program;

                let len_name = (crate::utils::guest_strlen(&memory, name as u64) + 1) as usize;
                let mut vec_name: Vec<i8> = memory.read_vec::<i8>(name as u64, len_name as u64)?;
                let converted_name = vec_name.as_mut_ptr() as *const i8;

                let result = unsafe {
                    (gl.proc_addresses.glGetFragDataLocation)(
                        converted_program,
                        converted_name,
                    ) 
                };

                Ok(result.into())
            },
        ),
    );
    exports.insert(
    "glGetFramebufferAttachmentParameteriv",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            target: u32,
            attachment: u32,
            pname: u32,
            params: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let mut gl = gl_arc.borrow_mut();
                let memory =
                    MemoryHandle::new(store.data().lazy.get().unwrap().memory.clone(), &store);
                let converted_target = target;
                let converted_attachment = attachment;
                let converted_pname = pname;

                let len_params = (crate::compsize::glGetFramebufferAttachmentParameteriv_params_compsize(&mut gl, crate::ffi::GLEnumGroupFramebufferAttachmentParameterName::from_raw(pname))) as usize;
                let mut vec_params: Vec<std::os::raw::c_int> = memory.read_vec::<i32>(params as u64, len_params as u64)?.iter().map(|v| *v as std::os::raw::c_int).collect::<Vec<_>>();
                let converted_params = vec_params.as_mut_ptr() as *mut std::os::raw::c_int;

                let result = unsafe {
                    (gl.proc_addresses.glGetFramebufferAttachmentParameteriv)(
                        converted_target,
                        converted_attachment,
                        converted_pname,
                        converted_params,
                    ) 
                };

                memory.write_slice::<i32>(params as u64, &vec_params.iter().map(|v| *v as _).collect::<Vec<_>>())?;

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glGetInteger64i_v",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            target: u32,
            index: u32,
            data: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let mut gl = gl_arc.borrow_mut();
                let memory =
                    MemoryHandle::new(store.data().lazy.get().unwrap().memory.clone(), &store);
                let converted_target = target;
                let converted_index = index;

                let len_data = (crate::compsize::glGetInteger64i_v_data_compsize(&mut gl, crate::ffi::GLEnumGroupGetPName::from_raw(target))) as usize;
                let mut vec_data: Vec<i64> = memory.read_vec::<i64>(data as u64, len_data as u64)?;
                let converted_data = vec_data.as_mut_ptr() as *mut i64;

                let result = unsafe {
                    (gl.proc_addresses.glGetInteger64i_v)(
                        converted_target,
                        converted_index,
                        converted_data,
                    ) 
                };

                memory.write_slice::<i64>(data as u64, &vec_data)?;

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glGetInteger64v",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            pname: u32,
            data: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let mut gl = gl_arc.borrow_mut();
                let memory =
                    MemoryHandle::new(store.data().lazy.get().unwrap().memory.clone(), &store);
                let converted_pname = pname;

                let len_data = (crate::compsize::glGetInteger64v_data_compsize(&mut gl, crate::ffi::GLEnumGroupGetPName::from_raw(pname))) as usize;
                let mut vec_data: Vec<i64> = memory.read_vec::<i64>(data as u64, len_data as u64)?;
                let converted_data = vec_data.as_mut_ptr() as *mut i64;

                let result = unsafe {
                    (gl.proc_addresses.glGetInteger64v)(
                        converted_pname,
                        converted_data,
                    ) 
                };

                memory.write_slice::<i64>(data as u64, &vec_data)?;

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glGetIntegeri_v",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            target: u32,
            index: u32,
            data: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let mut gl = gl_arc.borrow_mut();
                let memory =
                    MemoryHandle::new(store.data().lazy.get().unwrap().memory.clone(), &store);
                let converted_target = target;
                let converted_index = index;

                let len_data = (crate::compsize::glGetIntegeri_v_data_compsize(&mut gl, crate::ffi::GLEnumGroupGetPName::from_raw(target))) as usize;
                let mut vec_data: Vec<std::os::raw::c_int> = memory.read_vec::<i32>(data as u64, len_data as u64)?.iter().map(|v| *v as std::os::raw::c_int).collect::<Vec<_>>();
                let converted_data = vec_data.as_mut_ptr() as *mut std::os::raw::c_int;

                let result = unsafe {
                    (gl.proc_addresses.glGetIntegeri_v)(
                        converted_target,
                        converted_index,
                        converted_data,
                    ) 
                };

                memory.write_slice::<i32>(data as u64, &vec_data.iter().map(|v| *v as _).collect::<Vec<_>>())?;

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glGetIntegerv",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            pname: u32,
            data: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let mut gl = gl_arc.borrow_mut();
                let memory =
                    MemoryHandle::new(store.data().lazy.get().unwrap().memory.clone(), &store);
                let converted_pname = pname;

                let len_data = (crate::compsize::glGetIntegerv_data_compsize(&mut gl, crate::ffi::GLEnumGroupGetPName::from_raw(pname))) as usize;
                let mut vec_data: Vec<std::os::raw::c_int> = memory.read_vec::<i32>(data as u64, len_data as u64)?.iter().map(|v| *v as std::os::raw::c_int).collect::<Vec<_>>();
                let converted_data = vec_data.as_mut_ptr() as *mut std::os::raw::c_int;

                let result = unsafe {
                    (gl.proc_addresses.glGetIntegerv)(
                        converted_pname,
                        converted_data,
                    ) 
                };

                memory.write_slice::<i32>(data as u64, &vec_data.iter().map(|v| *v as _).collect::<Vec<_>>())?;

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glGetInternalformativ",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            target: u32,
            internalformat: u32,
            pname: u32,
            count: i32,
            params: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let memory =
                    MemoryHandle::new(store.data().lazy.get().unwrap().memory.clone(), &store);
                let converted_target = target;
                let converted_internalformat = internalformat;
                let converted_pname = pname;
                let converted_count = count;

                let len_params = (count) as usize;
                let mut vec_params: Vec<std::os::raw::c_int> = memory.read_vec::<i32>(params as u64, len_params as u64)?.iter().map(|v| *v as std::os::raw::c_int).collect::<Vec<_>>();
                let converted_params = vec_params.as_mut_ptr() as *mut std::os::raw::c_int;

                let result = unsafe {
                    (gl.proc_addresses.glGetInternalformativ)(
                        converted_target,
                        converted_internalformat,
                        converted_pname,
                        converted_count,
                        converted_params,
                    ) 
                };

                memory.write_slice::<i32>(params as u64, &vec_params.iter().map(|v| *v as _).collect::<Vec<_>>())?;

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glGetProgramBinary",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            program: u32,
            bufSize: i32,
            length: u32,
            binaryFormat: u32,
            binary: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let memory =
                    MemoryHandle::new(store.data().lazy.get().unwrap().memory.clone(), &store);
                let converted_program = program;
                let converted_bufSize = bufSize;

                let len_length = (1) as usize;
                let mut vec_length: Vec<std::os::raw::c_int> = memory.read_vec::<i32>(length as u64, len_length as u64)?.iter().map(|v| *v as std::os::raw::c_int).collect::<Vec<_>>();
                let converted_length = vec_length.as_mut_ptr() as *mut std::os::raw::c_int;


                let len_binaryFormat = (1) as usize;
                let mut vec_binaryFormat: Vec<std::os::raw::c_uint> = memory.read_vec::<u32>(binaryFormat as u64, len_binaryFormat as u64)?.iter().map(|v| *v as std::os::raw::c_uint).collect::<Vec<_>>();
                let converted_binaryFormat = vec_binaryFormat.as_mut_ptr() as *mut std::os::raw::c_uint;


                let len_binary = (bufSize) as usize;
                let mut vec_binary: Vec<u8> = memory.read_vec::<u8>(binary as u64, len_binary as u64)?;
                let converted_binary = vec_binary.as_mut_ptr() as *mut ();

                let result = unsafe {
                    (gl.proc_addresses.glGetProgramBinary)(
                        converted_program,
                        converted_bufSize,
                        converted_length,
                        converted_binaryFormat,
                        converted_binary,
                    ) 
                };

                memory.write_slice::<i32>(length as u64, &vec_length.iter().map(|v| *v as _).collect::<Vec<_>>())?;


                memory.write_slice::<u32>(binaryFormat as u64, &vec_binaryFormat.iter().map(|v| *v as _).collect::<Vec<_>>())?;


                memory.write_slice::<u8>(binary as u64, &vec_binary)?;

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glGetProgramInfoLog",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            program: u32,
            bufSize: i32,
            length: u32,
            infoLog: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let memory =
                    MemoryHandle::new(store.data().lazy.get().unwrap().memory.clone(), &store);
                let converted_program = program;
                let converted_bufSize = bufSize;

                let len_length = (1) as usize;
                let mut vec_length: Vec<std::os::raw::c_int> = memory.read_vec::<i32>(length as u64, len_length as u64)?.iter().map(|v| *v as std::os::raw::c_int).collect::<Vec<_>>();
                let converted_length = vec_length.as_mut_ptr() as *mut std::os::raw::c_int;


                let len_infoLog = (bufSize) as usize;
                let mut vec_infoLog: Vec<i8> = memory.read_vec::<i8>(infoLog as u64, len_infoLog as u64)?;
                let converted_infoLog = vec_infoLog.as_mut_ptr() as *mut i8;

                let result = unsafe {
                    (gl.proc_addresses.glGetProgramInfoLog)(
                        converted_program,
                        converted_bufSize,
                        converted_length,
                        converted_infoLog,
                    ) 
                };

                memory.write_slice::<i32>(length as u64, &vec_length.iter().map(|v| *v as _).collect::<Vec<_>>())?;


                memory.write_slice::<i8>(infoLog as u64, &vec_infoLog)?;

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glGetProgramiv",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            program: u32,
            pname: u32,
            params: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let mut gl = gl_arc.borrow_mut();
                let memory =
                    MemoryHandle::new(store.data().lazy.get().unwrap().memory.clone(), &store);
                let converted_program = program;
                let converted_pname = pname;

                let len_params = (crate::compsize::glGetProgramiv_params_compsize(&mut gl, crate::ffi::GLEnumGroupProgramPropertyARB::from_raw(pname))) as usize;
                let mut vec_params: Vec<std::os::raw::c_int> = memory.read_vec::<i32>(params as u64, len_params as u64)?.iter().map(|v| *v as std::os::raw::c_int).collect::<Vec<_>>();
                let converted_params = vec_params.as_mut_ptr() as *mut std::os::raw::c_int;

                let result = unsafe {
                    (gl.proc_addresses.glGetProgramiv)(
                        converted_program,
                        converted_pname,
                        converted_params,
                    ) 
                };

                memory.write_slice::<i32>(params as u64, &vec_params.iter().map(|v| *v as _).collect::<Vec<_>>())?;

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glGetQueryObjectuiv",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            id: u32,
            pname: u32,
            params: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let mut gl = gl_arc.borrow_mut();
                let memory =
                    MemoryHandle::new(store.data().lazy.get().unwrap().memory.clone(), &store);
                let converted_id = id;
                let converted_pname = pname;

                let len_params = (crate::compsize::glGetQueryObjectuiv_params_compsize(&mut gl, crate::ffi::GLEnumGroupQueryObjectParameterName::from_raw(pname))) as usize;
                let mut vec_params: Vec<std::os::raw::c_uint> = memory.read_vec::<u32>(params as u64, len_params as u64)?.iter().map(|v| *v as std::os::raw::c_uint).collect::<Vec<_>>();
                let converted_params = vec_params.as_mut_ptr() as *mut std::os::raw::c_uint;

                let result = unsafe {
                    (gl.proc_addresses.glGetQueryObjectuiv)(
                        converted_id,
                        converted_pname,
                        converted_params,
                    ) 
                };

                memory.write_slice::<u32>(params as u64, &vec_params.iter().map(|v| *v as _).collect::<Vec<_>>())?;

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glGetQueryiv",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            target: u32,
            pname: u32,
            params: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let mut gl = gl_arc.borrow_mut();
                let memory =
                    MemoryHandle::new(store.data().lazy.get().unwrap().memory.clone(), &store);
                let converted_target = target;
                let converted_pname = pname;

                let len_params = (crate::compsize::glGetQueryiv_params_compsize(&mut gl, crate::ffi::GLEnumGroupQueryParameterName::from_raw(pname))) as usize;
                let mut vec_params: Vec<std::os::raw::c_int> = memory.read_vec::<i32>(params as u64, len_params as u64)?.iter().map(|v| *v as std::os::raw::c_int).collect::<Vec<_>>();
                let converted_params = vec_params.as_mut_ptr() as *mut std::os::raw::c_int;

                let result = unsafe {
                    (gl.proc_addresses.glGetQueryiv)(
                        converted_target,
                        converted_pname,
                        converted_params,
                    ) 
                };

                memory.write_slice::<i32>(params as u64, &vec_params.iter().map(|v| *v as _).collect::<Vec<_>>())?;

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glGetRenderbufferParameteriv",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            target: u32,
            pname: u32,
            params: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let mut gl = gl_arc.borrow_mut();
                let memory =
                    MemoryHandle::new(store.data().lazy.get().unwrap().memory.clone(), &store);
                let converted_target = target;
                let converted_pname = pname;

                let len_params = (crate::compsize::glGetRenderbufferParameteriv_params_compsize(&mut gl, crate::ffi::GLEnumGroupRenderbufferParameterName::from_raw(pname))) as usize;
                let mut vec_params: Vec<std::os::raw::c_int> = memory.read_vec::<i32>(params as u64, len_params as u64)?.iter().map(|v| *v as std::os::raw::c_int).collect::<Vec<_>>();
                let converted_params = vec_params.as_mut_ptr() as *mut std::os::raw::c_int;

                let result = unsafe {
                    (gl.proc_addresses.glGetRenderbufferParameteriv)(
                        converted_target,
                        converted_pname,
                        converted_params,
                    ) 
                };

                memory.write_slice::<i32>(params as u64, &vec_params.iter().map(|v| *v as _).collect::<Vec<_>>())?;

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glGetSamplerParameterfv",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            sampler: u32,
            pname: u32,
            params: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let mut gl = gl_arc.borrow_mut();
                let memory =
                    MemoryHandle::new(store.data().lazy.get().unwrap().memory.clone(), &store);
                let converted_sampler = sampler;
                let converted_pname = pname;

                let len_params = (crate::compsize::glGetSamplerParameterfv_params_compsize(&mut gl, crate::ffi::GLEnumGroupSamplerParameterF::from_raw(pname))) as usize;
                let mut vec_params: Vec<f32> = memory.read_vec::<f32>(params as u64, len_params as u64)?;
                let converted_params = vec_params.as_mut_ptr() as *mut f32;

                let result = unsafe {
                    (gl.proc_addresses.glGetSamplerParameterfv)(
                        converted_sampler,
                        converted_pname,
                        converted_params,
                    ) 
                };

                memory.write_slice::<f32>(params as u64, &vec_params)?;

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glGetSamplerParameteriv",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            sampler: u32,
            pname: u32,
            params: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let mut gl = gl_arc.borrow_mut();
                let memory =
                    MemoryHandle::new(store.data().lazy.get().unwrap().memory.clone(), &store);
                let converted_sampler = sampler;
                let converted_pname = pname;

                let len_params = (crate::compsize::glGetSamplerParameteriv_params_compsize(&mut gl, crate::ffi::GLEnumGroupSamplerParameterI::from_raw(pname))) as usize;
                let mut vec_params: Vec<std::os::raw::c_int> = memory.read_vec::<i32>(params as u64, len_params as u64)?.iter().map(|v| *v as std::os::raw::c_int).collect::<Vec<_>>();
                let converted_params = vec_params.as_mut_ptr() as *mut std::os::raw::c_int;

                let result = unsafe {
                    (gl.proc_addresses.glGetSamplerParameteriv)(
                        converted_sampler,
                        converted_pname,
                        converted_params,
                    ) 
                };

                memory.write_slice::<i32>(params as u64, &vec_params.iter().map(|v| *v as _).collect::<Vec<_>>())?;

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glGetShaderInfoLog",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            shader: u32,
            bufSize: i32,
            length: u32,
            infoLog: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let memory =
                    MemoryHandle::new(store.data().lazy.get().unwrap().memory.clone(), &store);
                let converted_shader = shader;
                let converted_bufSize = bufSize;

                let len_length = (1) as usize;
                let mut vec_length: Vec<std::os::raw::c_int> = memory.read_vec::<i32>(length as u64, len_length as u64)?.iter().map(|v| *v as std::os::raw::c_int).collect::<Vec<_>>();
                let converted_length = vec_length.as_mut_ptr() as *mut std::os::raw::c_int;


                let len_infoLog = (bufSize) as usize;
                let mut vec_infoLog: Vec<i8> = memory.read_vec::<i8>(infoLog as u64, len_infoLog as u64)?;
                let converted_infoLog = vec_infoLog.as_mut_ptr() as *mut i8;

                let result = unsafe {
                    (gl.proc_addresses.glGetShaderInfoLog)(
                        converted_shader,
                        converted_bufSize,
                        converted_length,
                        converted_infoLog,
                    ) 
                };

                memory.write_slice::<i32>(length as u64, &vec_length.iter().map(|v| *v as _).collect::<Vec<_>>())?;


                memory.write_slice::<i8>(infoLog as u64, &vec_infoLog)?;

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glGetShaderPrecisionFormat",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            shadertype: u32,
            precisiontype: u32,
            range: u32,
            precision: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let memory =
                    MemoryHandle::new(store.data().lazy.get().unwrap().memory.clone(), &store);
                let converted_shadertype = shadertype;
                let converted_precisiontype = precisiontype;

                let len_range = (2) as usize;
                let mut vec_range: Vec<std::os::raw::c_int> = memory.read_vec::<i32>(range as u64, len_range as u64)?.iter().map(|v| *v as std::os::raw::c_int).collect::<Vec<_>>();
                let converted_range = vec_range.as_mut_ptr() as *mut std::os::raw::c_int;


                let len_precision = (1) as usize;
                let mut vec_precision: Vec<std::os::raw::c_int> = memory.read_vec::<i32>(precision as u64, len_precision as u64)?.iter().map(|v| *v as std::os::raw::c_int).collect::<Vec<_>>();
                let converted_precision = vec_precision.as_mut_ptr() as *mut std::os::raw::c_int;

                let result = unsafe {
                    (gl.proc_addresses.glGetShaderPrecisionFormat)(
                        converted_shadertype,
                        converted_precisiontype,
                        converted_range,
                        converted_precision,
                    ) 
                };

                memory.write_slice::<i32>(range as u64, &vec_range.iter().map(|v| *v as _).collect::<Vec<_>>())?;


                memory.write_slice::<i32>(precision as u64, &vec_precision.iter().map(|v| *v as _).collect::<Vec<_>>())?;

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glGetShaderSource",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            shader: u32,
            bufSize: i32,
            length: u32,
            source: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let memory =
                    MemoryHandle::new(store.data().lazy.get().unwrap().memory.clone(), &store);
                let converted_shader = shader;
                let converted_bufSize = bufSize;

                let len_length = (1) as usize;
                let mut vec_length: Vec<std::os::raw::c_int> = memory.read_vec::<i32>(length as u64, len_length as u64)?.iter().map(|v| *v as std::os::raw::c_int).collect::<Vec<_>>();
                let converted_length = vec_length.as_mut_ptr() as *mut std::os::raw::c_int;


                let len_source = (bufSize) as usize;
                let mut vec_source: Vec<i8> = memory.read_vec::<i8>(source as u64, len_source as u64)?;
                let converted_source = vec_source.as_mut_ptr() as *mut i8;

                let result = unsafe {
                    (gl.proc_addresses.glGetShaderSource)(
                        converted_shader,
                        converted_bufSize,
                        converted_length,
                        converted_source,
                    ) 
                };

                memory.write_slice::<i32>(length as u64, &vec_length.iter().map(|v| *v as _).collect::<Vec<_>>())?;


                memory.write_slice::<i8>(source as u64, &vec_source)?;

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glGetShaderiv",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            shader: u32,
            pname: u32,
            params: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let mut gl = gl_arc.borrow_mut();
                let memory =
                    MemoryHandle::new(store.data().lazy.get().unwrap().memory.clone(), &store);
                let converted_shader = shader;
                let converted_pname = pname;

                let len_params = (crate::compsize::glGetShaderiv_params_compsize(&mut gl, crate::ffi::GLEnumGroupShaderParameterName::from_raw(pname))) as usize;
                let mut vec_params: Vec<std::os::raw::c_int> = memory.read_vec::<i32>(params as u64, len_params as u64)?.iter().map(|v| *v as std::os::raw::c_int).collect::<Vec<_>>();
                let converted_params = vec_params.as_mut_ptr() as *mut std::os::raw::c_int;

                let result = unsafe {
                    (gl.proc_addresses.glGetShaderiv)(
                        converted_shader,
                        converted_pname,
                        converted_params,
                    ) 
                };

                memory.write_slice::<i32>(params as u64, &vec_params.iter().map(|v| *v as _).collect::<Vec<_>>())?;

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glGetSynciv",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            sync: u32,
            pname: u32,
            count: i32,
            length: u32,
            values: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let memory =
                    MemoryHandle::new(store.data().lazy.get().unwrap().memory.clone(), &store);
                let converted_sync = gl.resolve_opaque_sync_object(sync);
                let converted_pname = pname;
                let converted_count = count;

                let len_length = (1) as usize;
                let mut vec_length: Vec<std::os::raw::c_int> = memory.read_vec::<i32>(length as u64, len_length as u64)?.iter().map(|v| *v as std::os::raw::c_int).collect::<Vec<_>>();
                let converted_length = vec_length.as_mut_ptr() as *mut std::os::raw::c_int;


                let len_values = (count) as usize;
                let mut vec_values: Vec<std::os::raw::c_int> = memory.read_vec::<i32>(values as u64, len_values as u64)?.iter().map(|v| *v as std::os::raw::c_int).collect::<Vec<_>>();
                let converted_values = vec_values.as_mut_ptr() as *mut std::os::raw::c_int;

                let result = unsafe {
                    (gl.proc_addresses.glGetSynciv)(
                        converted_sync,
                        converted_pname,
                        converted_count,
                        converted_length,
                        converted_values,
                    ) 
                };

                memory.write_slice::<i32>(length as u64, &vec_length.iter().map(|v| *v as _).collect::<Vec<_>>())?;


                memory.write_slice::<i32>(values as u64, &vec_values.iter().map(|v| *v as _).collect::<Vec<_>>())?;

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glGetTexParameterfv",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            target: u32,
            pname: u32,
            params: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let mut gl = gl_arc.borrow_mut();
                let memory =
                    MemoryHandle::new(store.data().lazy.get().unwrap().memory.clone(), &store);
                let converted_target = target;
                let converted_pname = pname;

                let len_params = (crate::compsize::glGetTexParameterfv_params_compsize(&mut gl, crate::ffi::GLEnumGroupGetTextureParameter::from_raw(pname))) as usize;
                let mut vec_params: Vec<f32> = memory.read_vec::<f32>(params as u64, len_params as u64)?;
                let converted_params = vec_params.as_mut_ptr() as *mut f32;

                let result = unsafe {
                    (gl.proc_addresses.glGetTexParameterfv)(
                        converted_target,
                        converted_pname,
                        converted_params,
                    ) 
                };

                memory.write_slice::<f32>(params as u64, &vec_params)?;

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glGetTexParameteriv",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            target: u32,
            pname: u32,
            params: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let mut gl = gl_arc.borrow_mut();
                let memory =
                    MemoryHandle::new(store.data().lazy.get().unwrap().memory.clone(), &store);
                let converted_target = target;
                let converted_pname = pname;

                let len_params = (crate::compsize::glGetTexParameteriv_params_compsize(&mut gl, crate::ffi::GLEnumGroupGetTextureParameter::from_raw(pname))) as usize;
                let mut vec_params: Vec<std::os::raw::c_int> = memory.read_vec::<i32>(params as u64, len_params as u64)?.iter().map(|v| *v as std::os::raw::c_int).collect::<Vec<_>>();
                let converted_params = vec_params.as_mut_ptr() as *mut std::os::raw::c_int;

                let result = unsafe {
                    (gl.proc_addresses.glGetTexParameteriv)(
                        converted_target,
                        converted_pname,
                        converted_params,
                    ) 
                };

                memory.write_slice::<i32>(params as u64, &vec_params.iter().map(|v| *v as _).collect::<Vec<_>>())?;

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glGetTransformFeedbackVarying",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            program: u32,
            index: u32,
            bufSize: i32,
            length: u32,
            size: u32,
            _type: u32,
            name: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let memory =
                    MemoryHandle::new(store.data().lazy.get().unwrap().memory.clone(), &store);
                let converted_program = program;
                let converted_index = index;
                let converted_bufSize = bufSize;

                let len_length = (1) as usize;
                let mut vec_length: Vec<std::os::raw::c_int> = memory.read_vec::<i32>(length as u64, len_length as u64)?.iter().map(|v| *v as std::os::raw::c_int).collect::<Vec<_>>();
                let converted_length = vec_length.as_mut_ptr() as *mut std::os::raw::c_int;


                let len_size = (1) as usize;
                let mut vec_size: Vec<std::os::raw::c_int> = memory.read_vec::<i32>(size as u64, len_size as u64)?.iter().map(|v| *v as std::os::raw::c_int).collect::<Vec<_>>();
                let converted_size = vec_size.as_mut_ptr() as *mut std::os::raw::c_int;


                let len__type = (1) as usize;
                let mut vec__type: Vec<std::os::raw::c_uint> = memory.read_vec::<u32>(_type as u64, len__type as u64)?.iter().map(|v| *v as std::os::raw::c_uint).collect::<Vec<_>>();
                let converted__type = vec__type.as_mut_ptr() as *mut std::os::raw::c_uint;


                let len_name = (bufSize) as usize;
                let mut vec_name: Vec<i8> = memory.read_vec::<i8>(name as u64, len_name as u64)?;
                let converted_name = vec_name.as_mut_ptr() as *mut i8;

                let result = unsafe {
                    (gl.proc_addresses.glGetTransformFeedbackVarying)(
                        converted_program,
                        converted_index,
                        converted_bufSize,
                        converted_length,
                        converted_size,
                        converted__type,
                        converted_name,
                    ) 
                };

                memory.write_slice::<i32>(length as u64, &vec_length.iter().map(|v| *v as _).collect::<Vec<_>>())?;


                memory.write_slice::<i32>(size as u64, &vec_size.iter().map(|v| *v as _).collect::<Vec<_>>())?;


                memory.write_slice::<u32>(_type as u64, &vec__type.iter().map(|v| *v as _).collect::<Vec<_>>())?;


                memory.write_slice::<i8>(name as u64, &vec_name)?;

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glGetUniformBlockIndex",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            program: u32,
            uniformBlockName: u32,
        | -> Result<u32, wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let memory =
                    MemoryHandle::new(store.data().lazy.get().unwrap().memory.clone(), &store);
                let converted_program = program;

                let len_uniformBlockName = (crate::utils::guest_strlen(&memory, uniformBlockName as u64) + 1) as usize;
                let mut vec_uniformBlockName: Vec<i8> = memory.read_vec::<i8>(uniformBlockName as u64, len_uniformBlockName as u64)?;
                let converted_uniformBlockName = vec_uniformBlockName.as_mut_ptr() as *const i8;

                let result = unsafe {
                    (gl.proc_addresses.glGetUniformBlockIndex)(
                        converted_program,
                        converted_uniformBlockName,
                    ) 
                };

                Ok(result.into())
            },
        ),
    );
    exports.insert(
    "glGetUniformLocation",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            program: u32,
            name: u32,
        | -> Result<i32, wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let memory =
                    MemoryHandle::new(store.data().lazy.get().unwrap().memory.clone(), &store);
                let converted_program = program;

                let len_name = (crate::utils::guest_strlen(&memory, name as u64) + 1) as usize;
                let mut vec_name: Vec<i8> = memory.read_vec::<i8>(name as u64, len_name as u64)?;
                let converted_name = vec_name.as_mut_ptr() as *const i8;

                let result = unsafe {
                    (gl.proc_addresses.glGetUniformLocation)(
                        converted_program,
                        converted_name,
                    ) 
                };

                Ok(result.into())
            },
        ),
    );
    exports.insert(
    "glGetUniformfv",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            program: u32,
            location: i32,
            params: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let mut gl = gl_arc.borrow_mut();
                let memory =
                    MemoryHandle::new(store.data().lazy.get().unwrap().memory.clone(), &store);
                let converted_program = program;
                let converted_location = location;

                let len_params = (crate::compsize::glGetUniformfv_params_compsize(&mut gl, program,location)) as usize;
                let mut vec_params: Vec<f32> = memory.read_vec::<f32>(params as u64, len_params as u64)?;
                let converted_params = vec_params.as_mut_ptr() as *mut f32;

                let result = unsafe {
                    (gl.proc_addresses.glGetUniformfv)(
                        converted_program,
                        converted_location,
                        converted_params,
                    ) 
                };

                memory.write_slice::<f32>(params as u64, &vec_params)?;

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glGetUniformiv",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            program: u32,
            location: i32,
            params: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let mut gl = gl_arc.borrow_mut();
                let memory =
                    MemoryHandle::new(store.data().lazy.get().unwrap().memory.clone(), &store);
                let converted_program = program;
                let converted_location = location;

                let len_params = (crate::compsize::glGetUniformiv_params_compsize(&mut gl, program,location)) as usize;
                let mut vec_params: Vec<std::os::raw::c_int> = memory.read_vec::<i32>(params as u64, len_params as u64)?.iter().map(|v| *v as std::os::raw::c_int).collect::<Vec<_>>();
                let converted_params = vec_params.as_mut_ptr() as *mut std::os::raw::c_int;

                let result = unsafe {
                    (gl.proc_addresses.glGetUniformiv)(
                        converted_program,
                        converted_location,
                        converted_params,
                    ) 
                };

                memory.write_slice::<i32>(params as u64, &vec_params.iter().map(|v| *v as _).collect::<Vec<_>>())?;

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glGetUniformuiv",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            program: u32,
            location: i32,
            params: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let mut gl = gl_arc.borrow_mut();
                let memory =
                    MemoryHandle::new(store.data().lazy.get().unwrap().memory.clone(), &store);
                let converted_program = program;
                let converted_location = location;

                let len_params = (crate::compsize::glGetUniformuiv_params_compsize(&mut gl, program,location)) as usize;
                let mut vec_params: Vec<std::os::raw::c_uint> = memory.read_vec::<u32>(params as u64, len_params as u64)?.iter().map(|v| *v as std::os::raw::c_uint).collect::<Vec<_>>();
                let converted_params = vec_params.as_mut_ptr() as *mut std::os::raw::c_uint;

                let result = unsafe {
                    (gl.proc_addresses.glGetUniformuiv)(
                        converted_program,
                        converted_location,
                        converted_params,
                    ) 
                };

                memory.write_slice::<u32>(params as u64, &vec_params.iter().map(|v| *v as _).collect::<Vec<_>>())?;

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glGetVertexAttribIiv",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            index: u32,
            pname: u32,
            params: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let memory =
                    MemoryHandle::new(store.data().lazy.get().unwrap().memory.clone(), &store);
                let converted_index = index;
                let converted_pname = pname;

                let len_params = (1) as usize;
                let mut vec_params: Vec<std::os::raw::c_int> = memory.read_vec::<i32>(params as u64, len_params as u64)?.iter().map(|v| *v as std::os::raw::c_int).collect::<Vec<_>>();
                let converted_params = vec_params.as_mut_ptr() as *mut std::os::raw::c_int;

                let result = unsafe {
                    (gl.proc_addresses.glGetVertexAttribIiv)(
                        converted_index,
                        converted_pname,
                        converted_params,
                    ) 
                };

                memory.write_slice::<i32>(params as u64, &vec_params.iter().map(|v| *v as _).collect::<Vec<_>>())?;

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glGetVertexAttribIuiv",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            index: u32,
            pname: u32,
            params: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let memory =
                    MemoryHandle::new(store.data().lazy.get().unwrap().memory.clone(), &store);
                let converted_index = index;
                let converted_pname = pname;

                let len_params = (1) as usize;
                let mut vec_params: Vec<std::os::raw::c_uint> = memory.read_vec::<u32>(params as u64, len_params as u64)?.iter().map(|v| *v as std::os::raw::c_uint).collect::<Vec<_>>();
                let converted_params = vec_params.as_mut_ptr() as *mut std::os::raw::c_uint;

                let result = unsafe {
                    (gl.proc_addresses.glGetVertexAttribIuiv)(
                        converted_index,
                        converted_pname,
                        converted_params,
                    ) 
                };

                memory.write_slice::<u32>(params as u64, &vec_params.iter().map(|v| *v as _).collect::<Vec<_>>())?;

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glGetVertexAttribPointerv",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            index: u32,
            pname: u32,
            pointer: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let memory =
                    MemoryHandle::new(store.data().lazy.get().unwrap().memory.clone(), &store);
                let converted_index = index;
                let converted_pname = pname;

                let len_pointer = (1) as usize;
                let mut vec_pointer: Vec<*mut ()> = memory.read_vec::<u32>(pointer as u64, len_pointer as u64)?.iter().map(|v| *v as *mut ()).collect::<Vec<_>>();
                let converted_pointer = vec_pointer.as_mut_ptr() as *mut *mut ();

                let result = unsafe {
                    (gl.proc_addresses.glGetVertexAttribPointerv)(
                        converted_index,
                        converted_pname,
                        converted_pointer,
                    ) 
                };

                memory.write_slice::<u32>(pointer as u64, &vec_pointer.iter().map(|v| *v as _).collect::<Vec<_>>())?;

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glGetVertexAttribfv",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            index: u32,
            pname: u32,
            params: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let memory =
                    MemoryHandle::new(store.data().lazy.get().unwrap().memory.clone(), &store);
                let converted_index = index;
                let converted_pname = pname;

                let len_params = (4) as usize;
                let mut vec_params: Vec<f32> = memory.read_vec::<f32>(params as u64, len_params as u64)?;
                let converted_params = vec_params.as_mut_ptr() as *mut f32;

                let result = unsafe {
                    (gl.proc_addresses.glGetVertexAttribfv)(
                        converted_index,
                        converted_pname,
                        converted_params,
                    ) 
                };

                memory.write_slice::<f32>(params as u64, &vec_params)?;

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glGetVertexAttribiv",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            index: u32,
            pname: u32,
            params: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let memory =
                    MemoryHandle::new(store.data().lazy.get().unwrap().memory.clone(), &store);
                let converted_index = index;
                let converted_pname = pname;

                let len_params = (4) as usize;
                let mut vec_params: Vec<std::os::raw::c_int> = memory.read_vec::<i32>(params as u64, len_params as u64)?.iter().map(|v| *v as std::os::raw::c_int).collect::<Vec<_>>();
                let converted_params = vec_params.as_mut_ptr() as *mut std::os::raw::c_int;

                let result = unsafe {
                    (gl.proc_addresses.glGetVertexAttribiv)(
                        converted_index,
                        converted_pname,
                        converted_params,
                    ) 
                };

                memory.write_slice::<i32>(params as u64, &vec_params.iter().map(|v| *v as _).collect::<Vec<_>>())?;

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glHint",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            target: u32,
            mode: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let converted_target = target;
                let converted_mode = mode;
                let result = unsafe {
                    (gl.proc_addresses.glHint)(
                        converted_target,
                        converted_mode,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glInvalidateFramebuffer",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            target: u32,
            numAttachments: i32,
            attachments: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let memory =
                    MemoryHandle::new(store.data().lazy.get().unwrap().memory.clone(), &store);
                let converted_target = target;
                let converted_numAttachments = numAttachments;

                let len_attachments = (numAttachments) as usize;
                let mut vec_attachments: Vec<std::os::raw::c_uint> = memory.read_vec::<u32>(attachments as u64, len_attachments as u64)?.iter().map(|v| *v as std::os::raw::c_uint).collect::<Vec<_>>();
                let converted_attachments = vec_attachments.as_mut_ptr() as *const std::os::raw::c_uint;

                let result = unsafe {
                    (gl.proc_addresses.glInvalidateFramebuffer)(
                        converted_target,
                        converted_numAttachments,
                        converted_attachments,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glInvalidateSubFramebuffer",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            target: u32,
            numAttachments: i32,
            attachments: u32,
            x: i32,
            y: i32,
            width: i32,
            height: i32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let memory =
                    MemoryHandle::new(store.data().lazy.get().unwrap().memory.clone(), &store);
                let converted_target = target;
                let converted_numAttachments = numAttachments;

                let len_attachments = (numAttachments) as usize;
                let mut vec_attachments: Vec<std::os::raw::c_uint> = memory.read_vec::<u32>(attachments as u64, len_attachments as u64)?.iter().map(|v| *v as std::os::raw::c_uint).collect::<Vec<_>>();
                let converted_attachments = vec_attachments.as_mut_ptr() as *const std::os::raw::c_uint;

                let converted_x = x;
                let converted_y = y;
                let converted_width = width;
                let converted_height = height;
                let result = unsafe {
                    (gl.proc_addresses.glInvalidateSubFramebuffer)(
                        converted_target,
                        converted_numAttachments,
                        converted_attachments,
                        converted_x,
                        converted_y,
                        converted_width,
                        converted_height,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glIsBuffer",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            buffer: u32,
        | -> Result<u32, wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let converted_buffer = buffer;
                let result = unsafe {
                    (gl.proc_addresses.glIsBuffer)(
                        converted_buffer,
                    ) 
                };

                Ok(result.into())
            },
        ),
    );
    exports.insert(
    "glIsEnabled",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            cap: u32,
        | -> Result<u32, wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let converted_cap = cap;
                let result = unsafe {
                    (gl.proc_addresses.glIsEnabled)(
                        converted_cap,
                    ) 
                };

                Ok(result.into())
            },
        ),
    );
    exports.insert(
    "glIsFramebuffer",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            framebuffer: u32,
        | -> Result<u32, wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let converted_framebuffer = framebuffer;
                let result = unsafe {
                    (gl.proc_addresses.glIsFramebuffer)(
                        converted_framebuffer,
                    ) 
                };

                Ok(result.into())
            },
        ),
    );
    exports.insert(
    "glIsProgram",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            program: u32,
        | -> Result<u32, wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let converted_program = program;
                let result = unsafe {
                    (gl.proc_addresses.glIsProgram)(
                        converted_program,
                    ) 
                };

                Ok(result.into())
            },
        ),
    );
    exports.insert(
    "glIsQuery",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            id: u32,
        | -> Result<u32, wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let converted_id = id;
                let result = unsafe {
                    (gl.proc_addresses.glIsQuery)(
                        converted_id,
                    ) 
                };

                Ok(result.into())
            },
        ),
    );
    exports.insert(
    "glIsRenderbuffer",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            renderbuffer: u32,
        | -> Result<u32, wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let converted_renderbuffer = renderbuffer;
                let result = unsafe {
                    (gl.proc_addresses.glIsRenderbuffer)(
                        converted_renderbuffer,
                    ) 
                };

                Ok(result.into())
            },
        ),
    );
    exports.insert(
    "glIsSampler",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            sampler: u32,
        | -> Result<u32, wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let converted_sampler = sampler;
                let result = unsafe {
                    (gl.proc_addresses.glIsSampler)(
                        converted_sampler,
                    ) 
                };

                Ok(result.into())
            },
        ),
    );
    exports.insert(
    "glIsShader",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            shader: u32,
        | -> Result<u32, wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let converted_shader = shader;
                let result = unsafe {
                    (gl.proc_addresses.glIsShader)(
                        converted_shader,
                    ) 
                };

                Ok(result.into())
            },
        ),
    );
    exports.insert(
    "glIsSync",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            sync: u32,
        | -> Result<u32, wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let converted_sync = gl.resolve_opaque_sync_object(sync);
                let result = unsafe {
                    (gl.proc_addresses.glIsSync)(
                        converted_sync,
                    ) 
                };

                Ok(result.into())
            },
        ),
    );
    exports.insert(
    "glIsTexture",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            texture: u32,
        | -> Result<u32, wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let converted_texture = texture;
                let result = unsafe {
                    (gl.proc_addresses.glIsTexture)(
                        converted_texture,
                    ) 
                };

                Ok(result.into())
            },
        ),
    );
    exports.insert(
    "glIsTransformFeedback",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            id: u32,
        | -> Result<u32, wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let converted_id = id;
                let result = unsafe {
                    (gl.proc_addresses.glIsTransformFeedback)(
                        converted_id,
                    ) 
                };

                Ok(result.into())
            },
        ),
    );
    exports.insert(
    "glIsVertexArray",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            array: u32,
        | -> Result<u32, wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let converted_array = array;
                let result = unsafe {
                    (gl.proc_addresses.glIsVertexArray)(
                        converted_array,
                    ) 
                };

                Ok(result.into())
            },
        ),
    );
    exports.insert(
    "glIsVertexArrayOES",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            array: u32,
        | -> Result<u32, wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let converted_array = array;
                let result = unsafe {
                    (gl.proc_addresses.glIsVertexArrayOES)(
                        converted_array,
                    ) 
                };

                Ok(result.into())
            },
        ),
    );
    exports.insert(
    "glLineWidth",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            width: f32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let converted_width = width;
                let result = unsafe {
                    (gl.proc_addresses.glLineWidth)(
                        converted_width,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glLinkProgram",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            program: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let converted_program = program;
                let result = unsafe {
                    (gl.proc_addresses.glLinkProgram)(
                        converted_program,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glPauseTransformFeedback",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();

                let result = unsafe {
                    (gl.proc_addresses.glPauseTransformFeedback)(
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glPixelStorei",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            pname: u32,
            param: i32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let converted_pname = pname;
                let converted_param = param;
                let result = unsafe {
                    (gl.proc_addresses.glPixelStorei)(
                        converted_pname,
                        converted_param,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glPolygonOffset",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            factor: f32,
            units: f32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let converted_factor = factor;
                let converted_units = units;
                let result = unsafe {
                    (gl.proc_addresses.glPolygonOffset)(
                        converted_factor,
                        converted_units,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glProgramBinary",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            program: u32,
            binaryFormat: u32,
            binary: u32,
            length: i32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let memory =
                    MemoryHandle::new(store.data().lazy.get().unwrap().memory.clone(), &store);
                let converted_program = program;
                let converted_binaryFormat = binaryFormat;

                let len_binary = (length) as usize;
                let mut vec_binary: Vec<u8> = memory.read_vec::<u8>(binary as u64, len_binary as u64)?;
                let converted_binary = vec_binary.as_mut_ptr() as *const ();

                let converted_length = length;
                let result = unsafe {
                    (gl.proc_addresses.glProgramBinary)(
                        converted_program,
                        converted_binaryFormat,
                        converted_binary,
                        converted_length,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glProgramParameteri",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            program: u32,
            pname: u32,
            value: i32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let converted_program = program;
                let converted_pname = pname;
                let converted_value = value;
                let result = unsafe {
                    (gl.proc_addresses.glProgramParameteri)(
                        converted_program,
                        converted_pname,
                        converted_value,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glReadBuffer",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            src: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let converted_src = src;
                let result = unsafe {
                    (gl.proc_addresses.glReadBuffer)(
                        converted_src,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glReadPixels",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            x: i32,
            y: i32,
            width: i32,
            height: i32,
            format: u32,
            _type: u32,
            pixels: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let mut gl = gl_arc.borrow_mut();
                let memory =
                    MemoryHandle::new(store.data().lazy.get().unwrap().memory.clone(), &store);
                let converted_x = x;
                let converted_y = y;
                let converted_width = width;
                let converted_height = height;
                let converted_format = format;
                let converted__type = _type;

                let len_pixels = (crate::compsize::glReadPixels_pixels_compsize(&mut gl, crate::ffi::GLEnumGroupPixelFormat::from_raw(format),crate::ffi::GLEnumGroupPixelType::from_raw(_type),width,height)) as usize;
                let mut vec_pixels: Vec<u8> = memory.read_vec::<u8>(pixels as u64, len_pixels as u64)?;
                let converted_pixels = vec_pixels.as_mut_ptr() as *mut ();

                let result = unsafe {
                    (gl.proc_addresses.glReadPixels)(
                        converted_x,
                        converted_y,
                        converted_width,
                        converted_height,
                        converted_format,
                        converted__type,
                        converted_pixels,
                    ) 
                };

                memory.write_slice::<u8>(pixels as u64, &vec_pixels)?;

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glReleaseShaderCompiler",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();

                let result = unsafe {
                    (gl.proc_addresses.glReleaseShaderCompiler)(
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glRenderbufferStorage",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            target: u32,
            internalformat: u32,
            width: i32,
            height: i32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let converted_target = target;
                let converted_internalformat = internalformat;
                let converted_width = width;
                let converted_height = height;
                let result = unsafe {
                    (gl.proc_addresses.glRenderbufferStorage)(
                        converted_target,
                        converted_internalformat,
                        converted_width,
                        converted_height,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glRenderbufferStorageMultisample",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            target: u32,
            samples: i32,
            internalformat: u32,
            width: i32,
            height: i32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let converted_target = target;
                let converted_samples = samples;
                let converted_internalformat = internalformat;
                let converted_width = width;
                let converted_height = height;
                let result = unsafe {
                    (gl.proc_addresses.glRenderbufferStorageMultisample)(
                        converted_target,
                        converted_samples,
                        converted_internalformat,
                        converted_width,
                        converted_height,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glResumeTransformFeedback",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();

                let result = unsafe {
                    (gl.proc_addresses.glResumeTransformFeedback)(
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glSampleCoverage",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            value: f32,
            invert: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let converted_value = value;
                let converted_invert = invert as u8;
                let result = unsafe {
                    (gl.proc_addresses.glSampleCoverage)(
                        converted_value,
                        converted_invert,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glSamplerParameterf",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            sampler: u32,
            pname: u32,
            param: f32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let converted_sampler = sampler;
                let converted_pname = pname;
                let converted_param = param;
                let result = unsafe {
                    (gl.proc_addresses.glSamplerParameterf)(
                        converted_sampler,
                        converted_pname,
                        converted_param,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glSamplerParameterfv",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            sampler: u32,
            pname: u32,
            param: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let mut gl = gl_arc.borrow_mut();
                let memory =
                    MemoryHandle::new(store.data().lazy.get().unwrap().memory.clone(), &store);
                let converted_sampler = sampler;
                let converted_pname = pname;

                let len_param = (crate::compsize::glSamplerParameterfv_param_compsize(&mut gl, crate::ffi::GLEnumGroupSamplerParameterF::from_raw(pname))) as usize;
                let mut vec_param: Vec<f32> = memory.read_vec::<f32>(param as u64, len_param as u64)?;
                let converted_param = vec_param.as_mut_ptr() as *const f32;

                let result = unsafe {
                    (gl.proc_addresses.glSamplerParameterfv)(
                        converted_sampler,
                        converted_pname,
                        converted_param,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glSamplerParameteri",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            sampler: u32,
            pname: u32,
            param: i32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let converted_sampler = sampler;
                let converted_pname = pname;
                let converted_param = param;
                let result = unsafe {
                    (gl.proc_addresses.glSamplerParameteri)(
                        converted_sampler,
                        converted_pname,
                        converted_param,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glSamplerParameteriv",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            sampler: u32,
            pname: u32,
            param: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let mut gl = gl_arc.borrow_mut();
                let memory =
                    MemoryHandle::new(store.data().lazy.get().unwrap().memory.clone(), &store);
                let converted_sampler = sampler;
                let converted_pname = pname;

                let len_param = (crate::compsize::glSamplerParameteriv_param_compsize(&mut gl, crate::ffi::GLEnumGroupSamplerParameterI::from_raw(pname))) as usize;
                let mut vec_param: Vec<std::os::raw::c_int> = memory.read_vec::<i32>(param as u64, len_param as u64)?.iter().map(|v| *v as std::os::raw::c_int).collect::<Vec<_>>();
                let converted_param = vec_param.as_mut_ptr() as *const std::os::raw::c_int;

                let result = unsafe {
                    (gl.proc_addresses.glSamplerParameteriv)(
                        converted_sampler,
                        converted_pname,
                        converted_param,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glScissor",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            x: i32,
            y: i32,
            width: i32,
            height: i32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let converted_x = x;
                let converted_y = y;
                let converted_width = width;
                let converted_height = height;
                let result = unsafe {
                    (gl.proc_addresses.glScissor)(
                        converted_x,
                        converted_y,
                        converted_width,
                        converted_height,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glShaderBinary",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            count: i32,
            shaders: u32,
            binaryFormat: u32,
            binary: u32,
            length: i32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let memory =
                    MemoryHandle::new(store.data().lazy.get().unwrap().memory.clone(), &store);
                let converted_count = count;

                let len_shaders = (count) as usize;
                let mut vec_shaders: Vec<std::os::raw::c_uint> = memory.read_vec::<u32>(shaders as u64, len_shaders as u64)?.iter().map(|v| *v as std::os::raw::c_uint).collect::<Vec<_>>();
                let converted_shaders = vec_shaders.as_mut_ptr() as *const std::os::raw::c_uint;

                let converted_binaryFormat = binaryFormat;

                let len_binary = (length) as usize;
                let mut vec_binary: Vec<u8> = memory.read_vec::<u8>(binary as u64, len_binary as u64)?;
                let converted_binary = vec_binary.as_mut_ptr() as *const ();

                let converted_length = length;
                let result = unsafe {
                    (gl.proc_addresses.glShaderBinary)(
                        converted_count,
                        converted_shaders,
                        converted_binaryFormat,
                        converted_binary,
                        converted_length,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glStencilFunc",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            func: u32,
            _ref: i32,
            mask: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let converted_func = func;
                let converted__ref = _ref;
                let converted_mask = mask;
                let result = unsafe {
                    (gl.proc_addresses.glStencilFunc)(
                        converted_func,
                        converted__ref,
                        converted_mask,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glStencilFuncSeparate",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            face: u32,
            func: u32,
            _ref: i32,
            mask: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let converted_face = face;
                let converted_func = func;
                let converted__ref = _ref;
                let converted_mask = mask;
                let result = unsafe {
                    (gl.proc_addresses.glStencilFuncSeparate)(
                        converted_face,
                        converted_func,
                        converted__ref,
                        converted_mask,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glStencilMask",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            mask: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let converted_mask = mask;
                let result = unsafe {
                    (gl.proc_addresses.glStencilMask)(
                        converted_mask,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glStencilMaskSeparate",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            face: u32,
            mask: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let converted_face = face;
                let converted_mask = mask;
                let result = unsafe {
                    (gl.proc_addresses.glStencilMaskSeparate)(
                        converted_face,
                        converted_mask,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glStencilOp",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            fail: u32,
            zfail: u32,
            zpass: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let converted_fail = fail;
                let converted_zfail = zfail;
                let converted_zpass = zpass;
                let result = unsafe {
                    (gl.proc_addresses.glStencilOp)(
                        converted_fail,
                        converted_zfail,
                        converted_zpass,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glStencilOpSeparate",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            face: u32,
            sfail: u32,
            dpfail: u32,
            dppass: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let converted_face = face;
                let converted_sfail = sfail;
                let converted_dpfail = dpfail;
                let converted_dppass = dppass;
                let result = unsafe {
                    (gl.proc_addresses.glStencilOpSeparate)(
                        converted_face,
                        converted_sfail,
                        converted_dpfail,
                        converted_dppass,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glTexImage3D",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            target: u32,
            level: i32,
            internalformat: i32,
            width: i32,
            height: i32,
            depth: i32,
            border: i32,
            format: u32,
            _type: u32,
            pixels: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let mut gl = gl_arc.borrow_mut();
                let memory =
                    MemoryHandle::new(store.data().lazy.get().unwrap().memory.clone(), &store);
                let converted_target = target;
                let converted_level = level;
                let converted_internalformat = internalformat;
                let converted_width = width;
                let converted_height = height;
                let converted_depth = depth;
                let converted_border = border;
                let converted_format = format;
                let converted__type = _type;

                let len_pixels = (crate::compsize::glTexImage3D_pixels_compsize(&mut gl, crate::ffi::GLEnumGroupPixelFormat::from_raw(format),crate::ffi::GLEnumGroupPixelType::from_raw(_type),width,height,depth)) as usize;
                let mut vec_pixels: Vec<u8> = memory.read_vec::<u8>(pixels as u64, len_pixels as u64)?;
                let converted_pixels = vec_pixels.as_mut_ptr() as *const ();

                let result = unsafe {
                    (gl.proc_addresses.glTexImage3D)(
                        converted_target,
                        converted_level,
                        converted_internalformat,
                        converted_width,
                        converted_height,
                        converted_depth,
                        converted_border,
                        converted_format,
                        converted__type,
                        converted_pixels,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glTexParameterf",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            target: u32,
            pname: u32,
            param: f32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let converted_target = target;
                let converted_pname = pname;
                let converted_param = param;
                let result = unsafe {
                    (gl.proc_addresses.glTexParameterf)(
                        converted_target,
                        converted_pname,
                        converted_param,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glTexParameterfv",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            target: u32,
            pname: u32,
            params: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let mut gl = gl_arc.borrow_mut();
                let memory =
                    MemoryHandle::new(store.data().lazy.get().unwrap().memory.clone(), &store);
                let converted_target = target;
                let converted_pname = pname;

                let len_params = (crate::compsize::glTexParameterfv_params_compsize(&mut gl, crate::ffi::GLEnumGroupTextureParameterName::from_raw(pname))) as usize;
                let mut vec_params: Vec<f32> = memory.read_vec::<f32>(params as u64, len_params as u64)?;
                let converted_params = vec_params.as_mut_ptr() as *const f32;

                let result = unsafe {
                    (gl.proc_addresses.glTexParameterfv)(
                        converted_target,
                        converted_pname,
                        converted_params,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glTexParameteri",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            target: u32,
            pname: u32,
            param: i32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let converted_target = target;
                let converted_pname = pname;
                let converted_param = param;
                let result = unsafe {
                    (gl.proc_addresses.glTexParameteri)(
                        converted_target,
                        converted_pname,
                        converted_param,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glTexParameteriv",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            target: u32,
            pname: u32,
            params: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let mut gl = gl_arc.borrow_mut();
                let memory =
                    MemoryHandle::new(store.data().lazy.get().unwrap().memory.clone(), &store);
                let converted_target = target;
                let converted_pname = pname;

                let len_params = (crate::compsize::glTexParameteriv_params_compsize(&mut gl, crate::ffi::GLEnumGroupTextureParameterName::from_raw(pname))) as usize;
                let mut vec_params: Vec<std::os::raw::c_int> = memory.read_vec::<i32>(params as u64, len_params as u64)?.iter().map(|v| *v as std::os::raw::c_int).collect::<Vec<_>>();
                let converted_params = vec_params.as_mut_ptr() as *const std::os::raw::c_int;

                let result = unsafe {
                    (gl.proc_addresses.glTexParameteriv)(
                        converted_target,
                        converted_pname,
                        converted_params,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glTexStorage2D",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            target: u32,
            levels: i32,
            internalformat: u32,
            width: i32,
            height: i32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let converted_target = target;
                let converted_levels = levels;
                let converted_internalformat = internalformat;
                let converted_width = width;
                let converted_height = height;
                let result = unsafe {
                    (gl.proc_addresses.glTexStorage2D)(
                        converted_target,
                        converted_levels,
                        converted_internalformat,
                        converted_width,
                        converted_height,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glTexStorage3D",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            target: u32,
            levels: i32,
            internalformat: u32,
            width: i32,
            height: i32,
            depth: i32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let converted_target = target;
                let converted_levels = levels;
                let converted_internalformat = internalformat;
                let converted_width = width;
                let converted_height = height;
                let converted_depth = depth;
                let result = unsafe {
                    (gl.proc_addresses.glTexStorage3D)(
                        converted_target,
                        converted_levels,
                        converted_internalformat,
                        converted_width,
                        converted_height,
                        converted_depth,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glTexSubImage2D",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            target: u32,
            level: i32,
            xoffset: i32,
            yoffset: i32,
            width: i32,
            height: i32,
            format: u32,
            _type: u32,
            pixels: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let mut gl = gl_arc.borrow_mut();
                let memory =
                    MemoryHandle::new(store.data().lazy.get().unwrap().memory.clone(), &store);
                let converted_target = target;
                let converted_level = level;
                let converted_xoffset = xoffset;
                let converted_yoffset = yoffset;
                let converted_width = width;
                let converted_height = height;
                let converted_format = format;
                let converted__type = _type;

                let len_pixels = (crate::compsize::glTexSubImage2D_pixels_compsize(&mut gl, crate::ffi::GLEnumGroupPixelFormat::from_raw(format),crate::ffi::GLEnumGroupPixelType::from_raw(_type),width,height)) as usize;
                let mut vec_pixels: Vec<u8> = memory.read_vec::<u8>(pixels as u64, len_pixels as u64)?;
                let converted_pixels = vec_pixels.as_mut_ptr() as *const ();

                let result = unsafe {
                    (gl.proc_addresses.glTexSubImage2D)(
                        converted_target,
                        converted_level,
                        converted_xoffset,
                        converted_yoffset,
                        converted_width,
                        converted_height,
                        converted_format,
                        converted__type,
                        converted_pixels,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glTexSubImage3D",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            target: u32,
            level: i32,
            xoffset: i32,
            yoffset: i32,
            zoffset: i32,
            width: i32,
            height: i32,
            depth: i32,
            format: u32,
            _type: u32,
            pixels: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let mut gl = gl_arc.borrow_mut();
                let memory =
                    MemoryHandle::new(store.data().lazy.get().unwrap().memory.clone(), &store);
                let converted_target = target;
                let converted_level = level;
                let converted_xoffset = xoffset;
                let converted_yoffset = yoffset;
                let converted_zoffset = zoffset;
                let converted_width = width;
                let converted_height = height;
                let converted_depth = depth;
                let converted_format = format;
                let converted__type = _type;

                let len_pixels = (crate::compsize::glTexSubImage3D_pixels_compsize(&mut gl, crate::ffi::GLEnumGroupPixelFormat::from_raw(format),crate::ffi::GLEnumGroupPixelType::from_raw(_type),width,height,depth)) as usize;
                let mut vec_pixels: Vec<u8> = memory.read_vec::<u8>(pixels as u64, len_pixels as u64)?;
                let converted_pixels = vec_pixels.as_mut_ptr() as *const ();

                let result = unsafe {
                    (gl.proc_addresses.glTexSubImage3D)(
                        converted_target,
                        converted_level,
                        converted_xoffset,
                        converted_yoffset,
                        converted_zoffset,
                        converted_width,
                        converted_height,
                        converted_depth,
                        converted_format,
                        converted__type,
                        converted_pixels,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glTransformFeedbackVaryings",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            program: u32,
            count: i32,
            varyings: u32,
            bufferMode: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let memory =
                    MemoryHandle::new(store.data().lazy.get().unwrap().memory.clone(), &store);
                let converted_program = program;
                let converted_count = count;

                let len_varyings = (count) as usize;
                let mut vec_varyings: Vec<*const i8> = memory.read_vec::<u32>(varyings as u64, len_varyings as u64)?.iter().map(|v| *v as *const i8).collect::<Vec<_>>();
                let converted_varyings = vec_varyings.as_mut_ptr() as *const *const i8;

                let converted_bufferMode = bufferMode;
                let result = unsafe {
                    (gl.proc_addresses.glTransformFeedbackVaryings)(
                        converted_program,
                        converted_count,
                        converted_varyings,
                        converted_bufferMode,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glUniform1f",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            location: i32,
            v0: f32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let converted_location = location;
                let converted_v0 = v0;
                let result = unsafe {
                    (gl.proc_addresses.glUniform1f)(
                        converted_location,
                        converted_v0,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glUniform1fv",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            location: i32,
            count: i32,
            value: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let memory =
                    MemoryHandle::new(store.data().lazy.get().unwrap().memory.clone(), &store);
                let converted_location = location;
                let converted_count = count;

                let len_value = (count*1) as usize;
                let mut vec_value: Vec<f32> = memory.read_vec::<f32>(value as u64, len_value as u64)?;
                let converted_value = vec_value.as_mut_ptr() as *const f32;

                let result = unsafe {
                    (gl.proc_addresses.glUniform1fv)(
                        converted_location,
                        converted_count,
                        converted_value,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glUniform1i",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            location: i32,
            v0: i32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let converted_location = location;
                let converted_v0 = v0;
                let result = unsafe {
                    (gl.proc_addresses.glUniform1i)(
                        converted_location,
                        converted_v0,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glUniform1iv",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            location: i32,
            count: i32,
            value: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let memory =
                    MemoryHandle::new(store.data().lazy.get().unwrap().memory.clone(), &store);
                let converted_location = location;
                let converted_count = count;

                let len_value = (count*1) as usize;
                let mut vec_value: Vec<std::os::raw::c_int> = memory.read_vec::<i32>(value as u64, len_value as u64)?.iter().map(|v| *v as std::os::raw::c_int).collect::<Vec<_>>();
                let converted_value = vec_value.as_mut_ptr() as *const std::os::raw::c_int;

                let result = unsafe {
                    (gl.proc_addresses.glUniform1iv)(
                        converted_location,
                        converted_count,
                        converted_value,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glUniform1ui",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            location: i32,
            v0: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let converted_location = location;
                let converted_v0 = v0;
                let result = unsafe {
                    (gl.proc_addresses.glUniform1ui)(
                        converted_location,
                        converted_v0,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glUniform1uiv",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            location: i32,
            count: i32,
            value: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let memory =
                    MemoryHandle::new(store.data().lazy.get().unwrap().memory.clone(), &store);
                let converted_location = location;
                let converted_count = count;

                let len_value = (count*1) as usize;
                let mut vec_value: Vec<std::os::raw::c_uint> = memory.read_vec::<u32>(value as u64, len_value as u64)?.iter().map(|v| *v as std::os::raw::c_uint).collect::<Vec<_>>();
                let converted_value = vec_value.as_mut_ptr() as *const std::os::raw::c_uint;

                let result = unsafe {
                    (gl.proc_addresses.glUniform1uiv)(
                        converted_location,
                        converted_count,
                        converted_value,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glUniform2f",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            location: i32,
            v0: f32,
            v1: f32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let converted_location = location;
                let converted_v0 = v0;
                let converted_v1 = v1;
                let result = unsafe {
                    (gl.proc_addresses.glUniform2f)(
                        converted_location,
                        converted_v0,
                        converted_v1,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glUniform2fv",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            location: i32,
            count: i32,
            value: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let memory =
                    MemoryHandle::new(store.data().lazy.get().unwrap().memory.clone(), &store);
                let converted_location = location;
                let converted_count = count;

                let len_value = (count*2) as usize;
                let mut vec_value: Vec<f32> = memory.read_vec::<f32>(value as u64, len_value as u64)?;
                let converted_value = vec_value.as_mut_ptr() as *const f32;

                let result = unsafe {
                    (gl.proc_addresses.glUniform2fv)(
                        converted_location,
                        converted_count,
                        converted_value,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glUniform2i",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            location: i32,
            v0: i32,
            v1: i32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let converted_location = location;
                let converted_v0 = v0;
                let converted_v1 = v1;
                let result = unsafe {
                    (gl.proc_addresses.glUniform2i)(
                        converted_location,
                        converted_v0,
                        converted_v1,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glUniform2iv",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            location: i32,
            count: i32,
            value: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let memory =
                    MemoryHandle::new(store.data().lazy.get().unwrap().memory.clone(), &store);
                let converted_location = location;
                let converted_count = count;

                let len_value = (count*2) as usize;
                let mut vec_value: Vec<std::os::raw::c_int> = memory.read_vec::<i32>(value as u64, len_value as u64)?.iter().map(|v| *v as std::os::raw::c_int).collect::<Vec<_>>();
                let converted_value = vec_value.as_mut_ptr() as *const std::os::raw::c_int;

                let result = unsafe {
                    (gl.proc_addresses.glUniform2iv)(
                        converted_location,
                        converted_count,
                        converted_value,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glUniform2ui",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            location: i32,
            v0: u32,
            v1: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let converted_location = location;
                let converted_v0 = v0;
                let converted_v1 = v1;
                let result = unsafe {
                    (gl.proc_addresses.glUniform2ui)(
                        converted_location,
                        converted_v0,
                        converted_v1,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glUniform2uiv",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            location: i32,
            count: i32,
            value: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let memory =
                    MemoryHandle::new(store.data().lazy.get().unwrap().memory.clone(), &store);
                let converted_location = location;
                let converted_count = count;

                let len_value = (count*2) as usize;
                let mut vec_value: Vec<std::os::raw::c_uint> = memory.read_vec::<u32>(value as u64, len_value as u64)?.iter().map(|v| *v as std::os::raw::c_uint).collect::<Vec<_>>();
                let converted_value = vec_value.as_mut_ptr() as *const std::os::raw::c_uint;

                let result = unsafe {
                    (gl.proc_addresses.glUniform2uiv)(
                        converted_location,
                        converted_count,
                        converted_value,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glUniform3f",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            location: i32,
            v0: f32,
            v1: f32,
            v2: f32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let converted_location = location;
                let converted_v0 = v0;
                let converted_v1 = v1;
                let converted_v2 = v2;
                let result = unsafe {
                    (gl.proc_addresses.glUniform3f)(
                        converted_location,
                        converted_v0,
                        converted_v1,
                        converted_v2,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glUniform3fv",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            location: i32,
            count: i32,
            value: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let memory =
                    MemoryHandle::new(store.data().lazy.get().unwrap().memory.clone(), &store);
                let converted_location = location;
                let converted_count = count;

                let len_value = (count*3) as usize;
                let mut vec_value: Vec<f32> = memory.read_vec::<f32>(value as u64, len_value as u64)?;
                let converted_value = vec_value.as_mut_ptr() as *const f32;

                let result = unsafe {
                    (gl.proc_addresses.glUniform3fv)(
                        converted_location,
                        converted_count,
                        converted_value,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glUniform3i",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            location: i32,
            v0: i32,
            v1: i32,
            v2: i32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let converted_location = location;
                let converted_v0 = v0;
                let converted_v1 = v1;
                let converted_v2 = v2;
                let result = unsafe {
                    (gl.proc_addresses.glUniform3i)(
                        converted_location,
                        converted_v0,
                        converted_v1,
                        converted_v2,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glUniform3iv",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            location: i32,
            count: i32,
            value: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let memory =
                    MemoryHandle::new(store.data().lazy.get().unwrap().memory.clone(), &store);
                let converted_location = location;
                let converted_count = count;

                let len_value = (count*3) as usize;
                let mut vec_value: Vec<std::os::raw::c_int> = memory.read_vec::<i32>(value as u64, len_value as u64)?.iter().map(|v| *v as std::os::raw::c_int).collect::<Vec<_>>();
                let converted_value = vec_value.as_mut_ptr() as *const std::os::raw::c_int;

                let result = unsafe {
                    (gl.proc_addresses.glUniform3iv)(
                        converted_location,
                        converted_count,
                        converted_value,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glUniform3ui",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            location: i32,
            v0: u32,
            v1: u32,
            v2: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let converted_location = location;
                let converted_v0 = v0;
                let converted_v1 = v1;
                let converted_v2 = v2;
                let result = unsafe {
                    (gl.proc_addresses.glUniform3ui)(
                        converted_location,
                        converted_v0,
                        converted_v1,
                        converted_v2,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glUniform3uiv",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            location: i32,
            count: i32,
            value: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let memory =
                    MemoryHandle::new(store.data().lazy.get().unwrap().memory.clone(), &store);
                let converted_location = location;
                let converted_count = count;

                let len_value = (count*3) as usize;
                let mut vec_value: Vec<std::os::raw::c_uint> = memory.read_vec::<u32>(value as u64, len_value as u64)?.iter().map(|v| *v as std::os::raw::c_uint).collect::<Vec<_>>();
                let converted_value = vec_value.as_mut_ptr() as *const std::os::raw::c_uint;

                let result = unsafe {
                    (gl.proc_addresses.glUniform3uiv)(
                        converted_location,
                        converted_count,
                        converted_value,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glUniform4f",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            location: i32,
            v0: f32,
            v1: f32,
            v2: f32,
            v3: f32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let converted_location = location;
                let converted_v0 = v0;
                let converted_v1 = v1;
                let converted_v2 = v2;
                let converted_v3 = v3;
                let result = unsafe {
                    (gl.proc_addresses.glUniform4f)(
                        converted_location,
                        converted_v0,
                        converted_v1,
                        converted_v2,
                        converted_v3,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glUniform4fv",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            location: i32,
            count: i32,
            value: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let memory =
                    MemoryHandle::new(store.data().lazy.get().unwrap().memory.clone(), &store);
                let converted_location = location;
                let converted_count = count;

                let len_value = (count*4) as usize;
                let mut vec_value: Vec<f32> = memory.read_vec::<f32>(value as u64, len_value as u64)?;
                let converted_value = vec_value.as_mut_ptr() as *const f32;

                let result = unsafe {
                    (gl.proc_addresses.glUniform4fv)(
                        converted_location,
                        converted_count,
                        converted_value,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glUniform4i",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            location: i32,
            v0: i32,
            v1: i32,
            v2: i32,
            v3: i32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let converted_location = location;
                let converted_v0 = v0;
                let converted_v1 = v1;
                let converted_v2 = v2;
                let converted_v3 = v3;
                let result = unsafe {
                    (gl.proc_addresses.glUniform4i)(
                        converted_location,
                        converted_v0,
                        converted_v1,
                        converted_v2,
                        converted_v3,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glUniform4iv",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            location: i32,
            count: i32,
            value: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let memory =
                    MemoryHandle::new(store.data().lazy.get().unwrap().memory.clone(), &store);
                let converted_location = location;
                let converted_count = count;

                let len_value = (count*4) as usize;
                let mut vec_value: Vec<std::os::raw::c_int> = memory.read_vec::<i32>(value as u64, len_value as u64)?.iter().map(|v| *v as std::os::raw::c_int).collect::<Vec<_>>();
                let converted_value = vec_value.as_mut_ptr() as *const std::os::raw::c_int;

                let result = unsafe {
                    (gl.proc_addresses.glUniform4iv)(
                        converted_location,
                        converted_count,
                        converted_value,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glUniform4ui",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            location: i32,
            v0: u32,
            v1: u32,
            v2: u32,
            v3: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let converted_location = location;
                let converted_v0 = v0;
                let converted_v1 = v1;
                let converted_v2 = v2;
                let converted_v3 = v3;
                let result = unsafe {
                    (gl.proc_addresses.glUniform4ui)(
                        converted_location,
                        converted_v0,
                        converted_v1,
                        converted_v2,
                        converted_v3,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glUniform4uiv",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            location: i32,
            count: i32,
            value: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let memory =
                    MemoryHandle::new(store.data().lazy.get().unwrap().memory.clone(), &store);
                let converted_location = location;
                let converted_count = count;

                let len_value = (count*4) as usize;
                let mut vec_value: Vec<std::os::raw::c_uint> = memory.read_vec::<u32>(value as u64, len_value as u64)?.iter().map(|v| *v as std::os::raw::c_uint).collect::<Vec<_>>();
                let converted_value = vec_value.as_mut_ptr() as *const std::os::raw::c_uint;

                let result = unsafe {
                    (gl.proc_addresses.glUniform4uiv)(
                        converted_location,
                        converted_count,
                        converted_value,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glUniformBlockBinding",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            program: u32,
            uniformBlockIndex: u32,
            uniformBlockBinding: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let converted_program = program;
                let converted_uniformBlockIndex = uniformBlockIndex;
                let converted_uniformBlockBinding = uniformBlockBinding;
                let result = unsafe {
                    (gl.proc_addresses.glUniformBlockBinding)(
                        converted_program,
                        converted_uniformBlockIndex,
                        converted_uniformBlockBinding,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glUniformMatrix2fv",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            location: i32,
            count: i32,
            transpose: u32,
            value: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let memory =
                    MemoryHandle::new(store.data().lazy.get().unwrap().memory.clone(), &store);
                let converted_location = location;
                let converted_count = count;
                let converted_transpose = transpose as u8;

                let len_value = (count*4) as usize;
                let mut vec_value: Vec<f32> = memory.read_vec::<f32>(value as u64, len_value as u64)?;
                let converted_value = vec_value.as_mut_ptr() as *const f32;

                let result = unsafe {
                    (gl.proc_addresses.glUniformMatrix2fv)(
                        converted_location,
                        converted_count,
                        converted_transpose,
                        converted_value,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glUniformMatrix2x3fv",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            location: i32,
            count: i32,
            transpose: u32,
            value: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let memory =
                    MemoryHandle::new(store.data().lazy.get().unwrap().memory.clone(), &store);
                let converted_location = location;
                let converted_count = count;
                let converted_transpose = transpose as u8;

                let len_value = (count*6) as usize;
                let mut vec_value: Vec<f32> = memory.read_vec::<f32>(value as u64, len_value as u64)?;
                let converted_value = vec_value.as_mut_ptr() as *const f32;

                let result = unsafe {
                    (gl.proc_addresses.glUniformMatrix2x3fv)(
                        converted_location,
                        converted_count,
                        converted_transpose,
                        converted_value,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glUniformMatrix2x4fv",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            location: i32,
            count: i32,
            transpose: u32,
            value: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let memory =
                    MemoryHandle::new(store.data().lazy.get().unwrap().memory.clone(), &store);
                let converted_location = location;
                let converted_count = count;
                let converted_transpose = transpose as u8;

                let len_value = (count*8) as usize;
                let mut vec_value: Vec<f32> = memory.read_vec::<f32>(value as u64, len_value as u64)?;
                let converted_value = vec_value.as_mut_ptr() as *const f32;

                let result = unsafe {
                    (gl.proc_addresses.glUniformMatrix2x4fv)(
                        converted_location,
                        converted_count,
                        converted_transpose,
                        converted_value,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glUniformMatrix3fv",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            location: i32,
            count: i32,
            transpose: u32,
            value: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let memory =
                    MemoryHandle::new(store.data().lazy.get().unwrap().memory.clone(), &store);
                let converted_location = location;
                let converted_count = count;
                let converted_transpose = transpose as u8;

                let len_value = (count*9) as usize;
                let mut vec_value: Vec<f32> = memory.read_vec::<f32>(value as u64, len_value as u64)?;
                let converted_value = vec_value.as_mut_ptr() as *const f32;

                let result = unsafe {
                    (gl.proc_addresses.glUniformMatrix3fv)(
                        converted_location,
                        converted_count,
                        converted_transpose,
                        converted_value,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glUniformMatrix3x2fv",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            location: i32,
            count: i32,
            transpose: u32,
            value: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let memory =
                    MemoryHandle::new(store.data().lazy.get().unwrap().memory.clone(), &store);
                let converted_location = location;
                let converted_count = count;
                let converted_transpose = transpose as u8;

                let len_value = (count*6) as usize;
                let mut vec_value: Vec<f32> = memory.read_vec::<f32>(value as u64, len_value as u64)?;
                let converted_value = vec_value.as_mut_ptr() as *const f32;

                let result = unsafe {
                    (gl.proc_addresses.glUniformMatrix3x2fv)(
                        converted_location,
                        converted_count,
                        converted_transpose,
                        converted_value,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glUniformMatrix3x4fv",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            location: i32,
            count: i32,
            transpose: u32,
            value: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let memory =
                    MemoryHandle::new(store.data().lazy.get().unwrap().memory.clone(), &store);
                let converted_location = location;
                let converted_count = count;
                let converted_transpose = transpose as u8;

                let len_value = (count*12) as usize;
                let mut vec_value: Vec<f32> = memory.read_vec::<f32>(value as u64, len_value as u64)?;
                let converted_value = vec_value.as_mut_ptr() as *const f32;

                let result = unsafe {
                    (gl.proc_addresses.glUniformMatrix3x4fv)(
                        converted_location,
                        converted_count,
                        converted_transpose,
                        converted_value,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glUniformMatrix4fv",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            location: i32,
            count: i32,
            transpose: u32,
            value: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let memory =
                    MemoryHandle::new(store.data().lazy.get().unwrap().memory.clone(), &store);
                let converted_location = location;
                let converted_count = count;
                let converted_transpose = transpose as u8;

                let len_value = (count*16) as usize;
                let mut vec_value: Vec<f32> = memory.read_vec::<f32>(value as u64, len_value as u64)?;
                let converted_value = vec_value.as_mut_ptr() as *const f32;

                let result = unsafe {
                    (gl.proc_addresses.glUniformMatrix4fv)(
                        converted_location,
                        converted_count,
                        converted_transpose,
                        converted_value,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glUniformMatrix4x2fv",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            location: i32,
            count: i32,
            transpose: u32,
            value: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let memory =
                    MemoryHandle::new(store.data().lazy.get().unwrap().memory.clone(), &store);
                let converted_location = location;
                let converted_count = count;
                let converted_transpose = transpose as u8;

                let len_value = (count*8) as usize;
                let mut vec_value: Vec<f32> = memory.read_vec::<f32>(value as u64, len_value as u64)?;
                let converted_value = vec_value.as_mut_ptr() as *const f32;

                let result = unsafe {
                    (gl.proc_addresses.glUniformMatrix4x2fv)(
                        converted_location,
                        converted_count,
                        converted_transpose,
                        converted_value,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glUniformMatrix4x3fv",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            location: i32,
            count: i32,
            transpose: u32,
            value: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let memory =
                    MemoryHandle::new(store.data().lazy.get().unwrap().memory.clone(), &store);
                let converted_location = location;
                let converted_count = count;
                let converted_transpose = transpose as u8;

                let len_value = (count*12) as usize;
                let mut vec_value: Vec<f32> = memory.read_vec::<f32>(value as u64, len_value as u64)?;
                let converted_value = vec_value.as_mut_ptr() as *const f32;

                let result = unsafe {
                    (gl.proc_addresses.glUniformMatrix4x3fv)(
                        converted_location,
                        converted_count,
                        converted_transpose,
                        converted_value,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glUnmapBuffer",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            target: u32,
        | -> Result<u32, wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let converted_target = target;
                let result = unsafe {
                    (gl.proc_addresses.glUnmapBuffer)(
                        converted_target,
                    ) 
                };

                Ok(result.into())
            },
        ),
    );
    exports.insert(
    "glUseProgram",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            program: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let converted_program = program;
                let result = unsafe {
                    (gl.proc_addresses.glUseProgram)(
                        converted_program,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glValidateProgram",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            program: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let converted_program = program;
                let result = unsafe {
                    (gl.proc_addresses.glValidateProgram)(
                        converted_program,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glVertexAttrib1f",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            index: u32,
            x: f32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let converted_index = index;
                let converted_x = x;
                let result = unsafe {
                    (gl.proc_addresses.glVertexAttrib1f)(
                        converted_index,
                        converted_x,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glVertexAttrib1fv",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            index: u32,
            v: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let memory =
                    MemoryHandle::new(store.data().lazy.get().unwrap().memory.clone(), &store);
                let converted_index = index;

                let len_v = (1) as usize;
                let mut vec_v: Vec<f32> = memory.read_vec::<f32>(v as u64, len_v as u64)?;
                let converted_v = vec_v.as_mut_ptr() as *const f32;

                let result = unsafe {
                    (gl.proc_addresses.glVertexAttrib1fv)(
                        converted_index,
                        converted_v,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glVertexAttrib2f",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            index: u32,
            x: f32,
            y: f32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let converted_index = index;
                let converted_x = x;
                let converted_y = y;
                let result = unsafe {
                    (gl.proc_addresses.glVertexAttrib2f)(
                        converted_index,
                        converted_x,
                        converted_y,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glVertexAttrib2fv",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            index: u32,
            v: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let memory =
                    MemoryHandle::new(store.data().lazy.get().unwrap().memory.clone(), &store);
                let converted_index = index;

                let len_v = (2) as usize;
                let mut vec_v: Vec<f32> = memory.read_vec::<f32>(v as u64, len_v as u64)?;
                let converted_v = vec_v.as_mut_ptr() as *const f32;

                let result = unsafe {
                    (gl.proc_addresses.glVertexAttrib2fv)(
                        converted_index,
                        converted_v,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glVertexAttrib3f",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            index: u32,
            x: f32,
            y: f32,
            z: f32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let converted_index = index;
                let converted_x = x;
                let converted_y = y;
                let converted_z = z;
                let result = unsafe {
                    (gl.proc_addresses.glVertexAttrib3f)(
                        converted_index,
                        converted_x,
                        converted_y,
                        converted_z,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glVertexAttrib3fv",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            index: u32,
            v: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let memory =
                    MemoryHandle::new(store.data().lazy.get().unwrap().memory.clone(), &store);
                let converted_index = index;

                let len_v = (3) as usize;
                let mut vec_v: Vec<f32> = memory.read_vec::<f32>(v as u64, len_v as u64)?;
                let converted_v = vec_v.as_mut_ptr() as *const f32;

                let result = unsafe {
                    (gl.proc_addresses.glVertexAttrib3fv)(
                        converted_index,
                        converted_v,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glVertexAttrib4f",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            index: u32,
            x: f32,
            y: f32,
            z: f32,
            w: f32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let converted_index = index;
                let converted_x = x;
                let converted_y = y;
                let converted_z = z;
                let converted_w = w;
                let result = unsafe {
                    (gl.proc_addresses.glVertexAttrib4f)(
                        converted_index,
                        converted_x,
                        converted_y,
                        converted_z,
                        converted_w,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glVertexAttrib4fv",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            index: u32,
            v: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let memory =
                    MemoryHandle::new(store.data().lazy.get().unwrap().memory.clone(), &store);
                let converted_index = index;

                let len_v = (4) as usize;
                let mut vec_v: Vec<f32> = memory.read_vec::<f32>(v as u64, len_v as u64)?;
                let converted_v = vec_v.as_mut_ptr() as *const f32;

                let result = unsafe {
                    (gl.proc_addresses.glVertexAttrib4fv)(
                        converted_index,
                        converted_v,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glVertexAttribDivisor",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            index: u32,
            divisor: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let converted_index = index;
                let converted_divisor = divisor;
                let result = unsafe {
                    (gl.proc_addresses.glVertexAttribDivisor)(
                        converted_index,
                        converted_divisor,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glVertexAttribI4i",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            index: u32,
            x: i32,
            y: i32,
            z: i32,
            w: i32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let converted_index = index;
                let converted_x = x;
                let converted_y = y;
                let converted_z = z;
                let converted_w = w;
                let result = unsafe {
                    (gl.proc_addresses.glVertexAttribI4i)(
                        converted_index,
                        converted_x,
                        converted_y,
                        converted_z,
                        converted_w,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glVertexAttribI4iv",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            index: u32,
            v: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let memory =
                    MemoryHandle::new(store.data().lazy.get().unwrap().memory.clone(), &store);
                let converted_index = index;

                let len_v = (4) as usize;
                let mut vec_v: Vec<std::os::raw::c_int> = memory.read_vec::<i32>(v as u64, len_v as u64)?.iter().map(|v| *v as std::os::raw::c_int).collect::<Vec<_>>();
                let converted_v = vec_v.as_mut_ptr() as *const std::os::raw::c_int;

                let result = unsafe {
                    (gl.proc_addresses.glVertexAttribI4iv)(
                        converted_index,
                        converted_v,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glVertexAttribI4ui",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            index: u32,
            x: u32,
            y: u32,
            z: u32,
            w: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let converted_index = index;
                let converted_x = x;
                let converted_y = y;
                let converted_z = z;
                let converted_w = w;
                let result = unsafe {
                    (gl.proc_addresses.glVertexAttribI4ui)(
                        converted_index,
                        converted_x,
                        converted_y,
                        converted_z,
                        converted_w,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glVertexAttribI4uiv",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            index: u32,
            v: u32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let memory =
                    MemoryHandle::new(store.data().lazy.get().unwrap().memory.clone(), &store);
                let converted_index = index;

                let len_v = (4) as usize;
                let mut vec_v: Vec<std::os::raw::c_uint> = memory.read_vec::<u32>(v as u64, len_v as u64)?.iter().map(|v| *v as std::os::raw::c_uint).collect::<Vec<_>>();
                let converted_v = vec_v.as_mut_ptr() as *const std::os::raw::c_uint;

                let result = unsafe {
                    (gl.proc_addresses.glVertexAttribI4uiv)(
                        converted_index,
                        converted_v,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glViewport",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            x: i32,
            y: i32,
            width: i32,
            height: i32,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let converted_x = x;
                let converted_y = y;
                let converted_width = width;
                let converted_height = height;
                let result = unsafe {
                    (gl.proc_addresses.glViewport)(
                        converted_x,
                        converted_y,
                        converted_width,
                        converted_height,
                    ) 
                };

                Ok(result)
            },
        ),
    );
    exports.insert(
    "glWaitSync",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,
            sync: u32,
            flags: u32,
            timeout: u64,
        | -> Result<(), wasmer::RuntimeError> {
                let gl_arc = store.data_mut().gl.clone();
                let gl = gl_arc.borrow_mut();
                let converted_sync = gl.resolve_opaque_sync_object(sync);
                let converted_flags = flags;
                let converted_timeout = timeout;
                let result = unsafe {
                    (gl.proc_addresses.glWaitSync)(
                        converted_sync,
                        converted_flags,
                        converted_timeout,
                    ) 
                };

                Ok(result)
            },
        ),
    );

}