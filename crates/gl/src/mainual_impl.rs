use crate::{context::Context, ffi};

#[allow(non_snake_case)]
pub fn glShaderSource(
    _memory_factory: &mut Box<dyn webrogue_runtime::MemoryFactory>,
    _context: &mut Context,
    shader: u32,
    count: i32,
    string: u32,
    length: u32,
) -> () {
    let memory = _memory_factory.make_memory();
    let mut sources: Vec<Vec<i8>> = vec![];
    sources.reserve(count as usize);
    for i in 0..count {
        let mut size = None;
        if length != 0 {
            let size_candidate = memory
                .read::<i32>(webrogue_runtime::wiggle::GuestPtr::<i32>::new(
                    length + (i as u32) * 4,
                ))
                .unwrap() as i32;
            if size_candidate >= 0 {
                size = Some(size_candidate);
            }
        }
        let string_offset = memory
            .read::<u32>(webrogue_runtime::wiggle::GuestPtr::<u32>::new(
                string + (i as u32) * 4,
            ))
            .unwrap() as u32;
        sources.push(match size {
            None => {
                let mut source: Vec<i8> = vec![];
                let mut byte_offset = 0;
                loop {
                    let byte = memory
                        .read::<i8>(webrogue_runtime::wiggle::GuestPtr::<i8>::new(
                            string_offset + byte_offset,
                        ))
                        .unwrap();
                    if byte == 0 {
                        break;
                    } else {
                        byte_offset += 1;
                        source.push(byte);
                    }
                }
                source
            }
            Some(size) => {
                let slice = memory
                    .as_cow(webrogue_runtime::wiggle::GuestPtr::<[u8]>::new((
                        string_offset as u32,
                        size as u32,
                    )))
                    .unwrap();
                let v = slice.to_vec();
                unsafe {
                    std::slice::from_raw_parts(v.as_ptr() as *const i8, v.len() as usize).to_vec()
                }
            }
        });
    }

    let mut converted_string: Vec<*const i8> = vec![];
    let mut converted_length: Vec<std::os::raw::c_int> = vec![];

    sources.iter_mut().for_each(|source| {
        converted_string.push(source.as_ptr());
        converted_length.push(source.len() as std::os::raw::c_int);
    });

    let result = unsafe {
        (_context.proc_addresses.glShaderSource)(
            shader as std::os::raw::c_uint,
            count as std::os::raw::c_int,
            converted_string.as_ptr(),
            converted_length.as_mut_ptr(),
        )
    };
    drop(sources);
    result
}

#[allow(non_snake_case)]
pub fn glVertexAttribPointer(
    _memory_factory: &mut Box<dyn webrogue_runtime::MemoryFactory>,
    _context: &mut Context,
    index: u32,
    size: i32,
    _type: u32,
    normalized: u32,
    stride: i32,
    pointer: u32,
) -> () {
    unsafe {
        (_context.proc_addresses.glVertexAttribPointer)(
            index,
            size,
            _type,
            normalized as u8,
            stride,
            pointer as *mut (),
        )
    };
}

#[allow(non_snake_case)]
pub fn glVertexAttribIPointer(
    _memory_factory: &mut Box<dyn webrogue_runtime::MemoryFactory>,
    _context: &mut Context,
    index: u32,
    size: i32,
    _type: u32,
    stride: i32,
    pointer: u32,
) -> () {
    unsafe {
        (_context.proc_addresses.glVertexAttribIPointer)(
            index,
            size,
            _type,
            stride,
            pointer as *mut (),
        )
    };
}

#[allow(non_snake_case)]
pub fn glTexImage2D(
    _memory_factory: &mut Box<dyn webrogue_runtime::MemoryFactory>,
    _context: &mut Context,
    target: u32,
    level: i32,
    internalformat: i32,
    width: i32,
    height: i32,
    border: i32,
    format: u32,
    _type: u32,
    pixels: u32,
) -> () {
    let memory = _memory_factory.make_memory();
    let converted_target = target;
    let converted_level = level;
    let converted_internalformat = internalformat;
    let converted_width = width;
    let converted_height = height;
    let converted_border = border;
    let converted_format = format;
    let converted__type = _type;
    let len_pixels = (crate::compsize::glTexImage2D_pixels_compsize(
        _context,
        crate::ffi::GLEnumGroupPixelFormat::from_raw(format),
        crate::ffi::GLEnumGroupPixelType::from_raw(_type),
        width,
        height,
    )) as usize;

    let slice = if pixels != 0 {
        Some(
            memory
                .as_cow(webrogue_runtime::wiggle::GuestPtr::<[u8]>::new((
                    pixels as u32,
                    len_pixels as u32,
                )))
                .unwrap(),
        )
    } else {
        None
    };

    let converted_pixels = match &slice {
        Some(slice) => slice.as_ptr() as *mut (),
        None => std::ptr::null_mut(),
    };

    let result = unsafe {
        (_context.proc_addresses.glTexImage2D)(
            converted_target,
            converted_level,
            converted_internalformat,
            converted_width,
            converted_height,
            converted_border,
            converted_format,
            converted__type,
            converted_pixels,
        )
    };
    drop(slice);
    result
}

fn get_string(_context: &mut Context, name: u32) -> Option<Vec<u8>> {
    let gl_str = unsafe { (_context.proc_addresses.glGetString)(name) };
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

fn get_string_i(_context: &mut Context, i: i32, name: u32) -> Option<Vec<u8>> {
    if i == -1 {
        get_string(_context, name)
    } else {
        // TODO check
        match get_string(_context, name) {
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

#[allow(non_snake_case)]
pub fn glGetStringData(
    _memory_factory: &mut Box<dyn webrogue_runtime::MemoryFactory>,
    _context: &mut Context,
    i: i32,
    name: u32,
    data_ptr: u32,
) {
    if let Some(s) = get_string_i(_context, i, name) {
        let mut memory = _memory_factory.make_memory();
        let ptr = webrogue_runtime::wiggle::GuestPtr::<[u8]>::new((data_ptr, s.len() as u32));
        let _ = memory.copy_from_slice(s.as_slice(), ptr);
    }
}
#[allow(non_snake_case)]
pub fn glGetStringLen(
    _memory_factory: &mut Box<dyn webrogue_runtime::MemoryFactory>,
    _context: &mut Context,
    i: i32,
    name: u32,
) -> i32 {
    match get_string_i(_context, i, name) {
        None => -1,
        Some(s) => s.len() as i32,
    }
}

#[allow(non_snake_case)]
pub fn glBufferSubData(
    _memory_factory: &mut Box<dyn webrogue_runtime::MemoryFactory>,
    _context: &mut Context,
    target: u32,
    offset: i32,
    size: i32,
    data: u32,
) -> () {
    let memory = _memory_factory.make_memory();
    let converted_target = target;
    let converted_offset = offset as isize;
    let converted_size = size as isize;
    let len_data = (size) as usize;
    let data_cow = memory
        .as_cow(webrogue_runtime::wiggle::GuestPtr::<[u8]>::new((
            data as u32,
            len_data as u32,
        )))
        .unwrap();
    let converted_data = data_cow.as_ptr();
    let result = unsafe {
        (_context.proc_addresses.glBufferSubData)(
            converted_target,
            converted_offset,
            converted_size,
            converted_data as *const (),
        )
    };
    drop(data_cow);
    result
}
#[allow(non_snake_case)]
pub fn glDrawElements(
    _memory_factory: &mut Box<dyn webrogue_runtime::MemoryFactory>,
    _context: &mut Context,
    mode: u32,
    count: i32,
    _type: u32,
    indices: u32,
) -> () {
    let memory = _memory_factory.make_memory();
    let mut element_array_buffer = 0;
    unsafe {
        (_context.proc_addresses.glGetIntegerv)(
            ffi::GL_ELEMENT_ARRAY_BUFFER_BINDING,
            &mut element_array_buffer,
        );
    }
    if element_array_buffer == 0 {
        let len_indices = (crate::compsize::glDrawElements_indices_compsize(
            _context,
            count,
            crate::ffi::GLEnumGroupDrawElementsType::from_raw(_type),
        )) as usize;
        let indices_cow = memory
            .as_cow(webrogue_runtime::wiggle::GuestPtr::<[u8]>::new((
                indices as u32,
                len_indices as u32,
            )))
            .unwrap();
        unsafe {
            (_context.proc_addresses.glDrawElements)(
                mode,
                count,
                _type,
                indices_cow.as_ptr() as *const (),
            )
        };
    } else {
        unsafe {
            (_context.proc_addresses.glDrawElements)(mode, count, _type, indices as *const ())
        };
    }
}
#[allow(non_snake_case)]
pub fn glDrawElementsInstanced(
    _memory_factory: &mut Box<dyn webrogue_runtime::MemoryFactory>,
    _context: &mut Context,
    mode: u32,
    count: i32,
    _type: u32,
    indices: u32,
    instancecount: i32,
) -> () {
    let memory = _memory_factory.make_memory();
    let mut element_array_buffer = 0;
    unsafe {
        (_context.proc_addresses.glGetIntegerv)(
            ffi::GL_ELEMENT_ARRAY_BUFFER_BINDING,
            &mut element_array_buffer,
        );
    }
    if element_array_buffer == 0 {
        let len_indices = (crate::compsize::glDrawElements_indices_compsize(
            _context,
            count,
            crate::ffi::GLEnumGroupDrawElementsType::from_raw(_type),
        )) as usize;
        let indices_cow = memory
            .as_cow(webrogue_runtime::wiggle::GuestPtr::<[u8]>::new((
                indices as u32,
                len_indices as u32,
            )))
            .unwrap();
        unsafe {
            (_context.proc_addresses.glDrawElementsInstanced)(
                mode,
                count,
                _type,
                indices_cow.as_ptr() as *const (),
                instancecount,
            )
        };
    } else {
        unsafe {
            (_context.proc_addresses.glDrawElementsInstanced)(
                mode,
                count,
                _type,
                indices as *const (),
                instancecount,
            )
        };
    }
}
#[allow(non_snake_case)]
pub fn glDrawRangeElements(
    _memory_factory: &mut Box<dyn webrogue_runtime::MemoryFactory>,
    _context: &mut Context,
    mode: u32,
    start: u32,
    end: u32,
    count: i32,
    _type: u32,
    indices: u32,
) -> () {
    let memory = _memory_factory.make_memory();
    let mut element_array_buffer = 0;
    unsafe {
        (_context.proc_addresses.glGetIntegerv)(
            ffi::GL_ELEMENT_ARRAY_BUFFER_BINDING,
            &mut element_array_buffer,
        );
    }
    if element_array_buffer == 0 {
        let len_indices = (crate::compsize::glDrawElements_indices_compsize(
            _context,
            count,
            crate::ffi::GLEnumGroupDrawElementsType::from_raw(_type),
        )) as usize;
        let indices_cow = memory
            .as_cow(webrogue_runtime::wiggle::GuestPtr::<[u8]>::new((
                indices as u32,
                len_indices as u32,
            )))
            .unwrap();
        unsafe {
            (_context.proc_addresses.glDrawRangeElements)(
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
            (_context.proc_addresses.glDrawRangeElements)(
                mode,
                start,
                end,
                count,
                _type,
                indices as *const (),
            )
        };
    }
}

// TODO check
#[allow(non_snake_case)]
pub fn glGetUniformIndices(
    _memory_factory: &mut Box<dyn webrogue_runtime::MemoryFactory>,
    _context: &mut Context,
    program: u32,
    uniformCount: i32,
    uniformNames: u32,
    uniformIndices: u32,
) -> () {
    let mut memory = _memory_factory.make_memory();
    let uniform_names_ptrs = memory
        .to_vec(webrogue_runtime::wiggle::GuestPtr::<[u32]>::new((
            uniformNames as u32,
            uniformCount as u32,
        )))
        .unwrap();
    let uniform_names = uniform_names_ptrs
        .iter()
        .map(|ptr| {
            memory
                .as_cow(webrogue_runtime::wiggle::GuestPtr::<[u8]>::new((
                    *ptr,
                    (crate::utils::guest_strlen(&memory, *ptr) + 1) as u32,
                )))
                .unwrap()
        })
        .collect::<Vec<_>>();
    let converted_uniform_names = uniform_names
        .iter()
        .map(|name| name.as_ptr() as *const i8)
        .collect::<Vec<_>>();

    let mut vec_uniformIndices: Vec<std::os::raw::c_uint> = vec![];
    vec_uniformIndices.reserve(uniformCount as usize);
    for i in 0..(uniformCount as u32) {
        vec_uniformIndices.push(
            memory
                .read::<u32>(webrogue_runtime::wiggle::GuestPtr::<u32>::new(
                    uniformIndices + i * 4,
                ))
                .unwrap() as std::os::raw::c_uint,
        );
    }
    let converted_uniformIndices = vec_uniformIndices.as_mut_ptr() as *mut std::os::raw::c_uint;
    let result = unsafe {
        (_context.proc_addresses.glGetUniformIndices)(
            program,
            uniformCount,
            converted_uniform_names.as_ptr(),
            converted_uniformIndices,
        )
    };
    for (i, value) in vec_uniformIndices.iter().enumerate() {
        memory
            .write::<u32>(
                webrogue_runtime::wiggle::GuestPtr::<u32>::new(uniformIndices + (i as u32) * 4),
                *value as u32,
            )
            .unwrap()
    }
    result
}
#[allow(non_snake_case)]
pub fn glMapBufferRange(
    _memory_factory: &mut Box<dyn webrogue_runtime::MemoryFactory>,
    _context: &mut Context,
    _target: u32,
    _offset: i32,
    _length: i32,
    _access: u32,
) -> u32 {
    todo!()
}

pub fn init_ptrs(
    _memory_factory: &mut Box<dyn webrogue_runtime::MemoryFactory>,
    _context: &mut Context,
) {
    _context
        .proc_addresses
        .fill(unsafe { _context.gfx_context.as_mut().unwrap() });
}
