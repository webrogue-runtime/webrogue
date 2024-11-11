use crate::ffi::{self};
use crate::gl::GL;

// GLEnumGroupBuffer

#[allow(non_snake_case)]
pub(crate) fn glClearBufferfv_value_compsize(
    _gl: &mut GL,
    buffer: Option<ffi::GLEnumGroupBuffer>,
) -> usize {
    match buffer {
        None => 1,
        Some(buffer) => match buffer {
            ffi::GLEnumGroupBuffer::GL_COLOR => 4,
            ffi::GLEnumGroupBuffer::GL_DEPTH => 1,
            ffi::GLEnumGroupBuffer::GL_STENCIL => 1,
        },
    }
}
#[allow(non_snake_case)]
pub(crate) fn glClearBufferiv_value_compsize(
    _gl: &mut GL,
    buffer: Option<ffi::GLEnumGroupBuffer>,
) -> usize {
    glClearBufferfv_value_compsize(_gl, buffer)
}
#[allow(non_snake_case)]
pub(crate) fn glClearBufferuiv_value_compsize(
    _gl: &mut GL,
    buffer: Option<ffi::GLEnumGroupBuffer>,
) -> usize {
    glClearBufferfv_value_compsize(_gl, buffer)
}

// GLEnumGroupUniformBlockPName

#[allow(non_snake_case)]
pub(crate) fn glGetActiveUniformBlockiv_params_compsize(
    _gl: &mut GL,
    program: u32,
    uniformBlockIndex: u32,
    pname: Option<ffi::GLEnumGroupUniformBlockPName>,
) -> usize {
    match pname {
        None => 1,
        Some(pname) => match pname {
            ffi::GLEnumGroupUniformBlockPName::GL_UNIFORM_BLOCK_ACTIVE_UNIFORMS => 1,
            ffi::GLEnumGroupUniformBlockPName::GL_UNIFORM_BLOCK_ACTIVE_UNIFORM_INDICES => {
                let mut indices: i32 = 0;
                unsafe {
                    (_gl.proc_addresses.glGetActiveUniformBlockiv)(
                        program,
                        uniformBlockIndex,
                        ffi::GL_UNIFORM_BLOCK_ACTIVE_UNIFORMS,
                        &mut indices,
                    );
                }
                indices as usize
            }
            ffi::GLEnumGroupUniformBlockPName::GL_UNIFORM_BLOCK_BINDING => 1,
            ffi::GLEnumGroupUniformBlockPName::GL_UNIFORM_BLOCK_DATA_SIZE => 1,
            ffi::GLEnumGroupUniformBlockPName::GL_UNIFORM_BLOCK_NAME_LENGTH => 1,
            ffi::GLEnumGroupUniformBlockPName::GL_UNIFORM_BLOCK_REFERENCED_BY_FRAGMENT_SHADER => 1,
            ffi::GLEnumGroupUniformBlockPName::GL_UNIFORM_BLOCK_REFERENCED_BY_VERTEX_SHADER => 1,
        },
    }
}

// GLEnumGroupUniformPName

#[allow(non_snake_case)]
pub(crate) fn glGetActiveUniformsiv_params_compsize(
    _gl: &mut GL,
    uniformCount: i32,
    pname: Option<ffi::GLEnumGroupUniformPName>,
) -> usize {
    let num_params = match pname {
        None => 1,
        Some(pname) => match pname {
            ffi::GLEnumGroupUniformPName::GL_UNIFORM_ARRAY_STRIDE => 1,
            ffi::GLEnumGroupUniformPName::GL_UNIFORM_BLOCK_INDEX => 1,
            ffi::GLEnumGroupUniformPName::GL_UNIFORM_IS_ROW_MAJOR => 1,
            ffi::GLEnumGroupUniformPName::GL_UNIFORM_MATRIX_STRIDE => 1,
            ffi::GLEnumGroupUniformPName::GL_UNIFORM_NAME_LENGTH => 1,
            ffi::GLEnumGroupUniformPName::GL_UNIFORM_OFFSET => 1,
            ffi::GLEnumGroupUniformPName::GL_UNIFORM_SIZE => 1,
            ffi::GLEnumGroupUniformPName::GL_UNIFORM_TYPE => 1,
        },
    };
    (num_params * uniformCount) as usize
}

// GLEnumGroupGetPName

#[allow(non_snake_case)]
pub(crate) fn glGetBooleanv_data_compsize(
    _gl: &mut GL,
    pname: Option<ffi::GLEnumGroupGetPName>,
) -> usize {
    match pname {
        None => 1,
        Some(pname) => match pname {
            ffi::GLEnumGroupGetPName::GL_ACTIVE_TEXTURE => 1,
            ffi::GLEnumGroupGetPName::GL_ALIASED_LINE_WIDTH_RANGE => 1,
            ffi::GLEnumGroupGetPName::GL_ALIASED_POINT_SIZE_RANGE => 1,
            ffi::GLEnumGroupGetPName::GL_ALPHA_BITS => 1,
            ffi::GLEnumGroupGetPName::GL_ARRAY_BUFFER_BINDING => 1,
            ffi::GLEnumGroupGetPName::GL_BLEND => 1,
            ffi::GLEnumGroupGetPName::GL_BLEND_COLOR => 4,
            ffi::GLEnumGroupGetPName::GL_BLEND_DST_ALPHA => 1,
            ffi::GLEnumGroupGetPName::GL_BLEND_DST_RGB => 1,
            ffi::GLEnumGroupGetPName::GL_BLEND_EQUATION => 1,
            ffi::GLEnumGroupGetPName::GL_BLEND_EQUATION_ALPHA => 1,
            ffi::GLEnumGroupGetPName::GL_BLEND_EQUATION_RGB => 1,
            ffi::GLEnumGroupGetPName::GL_BLEND_SRC_ALPHA => 1,
            ffi::GLEnumGroupGetPName::GL_BLEND_SRC_RGB => 1,
            ffi::GLEnumGroupGetPName::GL_BLUE_BITS => 1,
            ffi::GLEnumGroupGetPName::GL_COLOR_CLEAR_VALUE => 4,
            ffi::GLEnumGroupGetPName::GL_COLOR_WRITEMASK => 4,
            ffi::GLEnumGroupGetPName::GL_COMPRESSED_TEXTURE_FORMATS => {
                ffi::GL_NUM_COMPRESSED_TEXTURE_FORMATS as usize
            }
            ffi::GLEnumGroupGetPName::GL_CULL_FACE => 1,
            ffi::GLEnumGroupGetPName::GL_CULL_FACE_MODE => 1,
            ffi::GLEnumGroupGetPName::GL_CURRENT_PROGRAM => 1,
            ffi::GLEnumGroupGetPName::GL_DEPTH_BITS => 1,
            ffi::GLEnumGroupGetPName::GL_DEPTH_CLEAR_VALUE => 1,
            ffi::GLEnumGroupGetPName::GL_DEPTH_FUNC => 1,
            ffi::GLEnumGroupGetPName::GL_DEPTH_RANGE => 2,
            ffi::GLEnumGroupGetPName::GL_DEPTH_TEST => 1,
            ffi::GLEnumGroupGetPName::GL_DEPTH_WRITEMASK => 1,
            ffi::GLEnumGroupGetPName::GL_DITHER => 1,
            ffi::GLEnumGroupGetPName::GL_DRAW_FRAMEBUFFER_BINDING => 1,
            ffi::GLEnumGroupGetPName::GL_ELEMENT_ARRAY_BUFFER_BINDING => 1,
            ffi::GLEnumGroupGetPName::GL_FETCH_PER_SAMPLE_ARM => 1,
            ffi::GLEnumGroupGetPName::GL_FRAGMENT_SHADER_DERIVATIVE_HINT => 1,
            ffi::GLEnumGroupGetPName::GL_FRAGMENT_SHADER_FRAMEBUFFER_FETCH_MRT_ARM => 1,
            ffi::GLEnumGroupGetPName::GL_FRONT_FACE => 1,
            ffi::GLEnumGroupGetPName::GL_GREEN_BITS => 1,
            ffi::GLEnumGroupGetPName::GL_IMPLEMENTATION_COLOR_READ_FORMAT => 1,
            ffi::GLEnumGroupGetPName::GL_IMPLEMENTATION_COLOR_READ_TYPE => 1,
            ffi::GLEnumGroupGetPName::GL_LINE_WIDTH => 1,
            ffi::GLEnumGroupGetPName::GL_MAJOR_VERSION => 1,
            ffi::GLEnumGroupGetPName::GL_MAX_3D_TEXTURE_SIZE => 1,
            ffi::GLEnumGroupGetPName::GL_MAX_ARRAY_TEXTURE_LAYERS => 1,
            ffi::GLEnumGroupGetPName::GL_MAX_COLOR_ATTACHMENTS => 1,
            ffi::GLEnumGroupGetPName::GL_MAX_COLOR_ATTACHMENTS_NV => 1,
            ffi::GLEnumGroupGetPName::GL_MAX_COMBINED_FRAGMENT_UNIFORM_COMPONENTS => 1,
            ffi::GLEnumGroupGetPName::GL_MAX_COMBINED_TEXTURE_IMAGE_UNITS => 1,
            ffi::GLEnumGroupGetPName::GL_MAX_COMBINED_UNIFORM_BLOCKS => 1,
            ffi::GLEnumGroupGetPName::GL_MAX_COMBINED_VERTEX_UNIFORM_COMPONENTS => 1,
            ffi::GLEnumGroupGetPName::GL_MAX_CUBE_MAP_TEXTURE_SIZE => 1,
            ffi::GLEnumGroupGetPName::GL_MAX_DRAW_BUFFERS => 1,
            ffi::GLEnumGroupGetPName::GL_MAX_ELEMENTS_INDICES => 1,
            ffi::GLEnumGroupGetPName::GL_MAX_ELEMENTS_VERTICES => 1,
            ffi::GLEnumGroupGetPName::GL_MAX_ELEMENT_INDEX => 1,
            ffi::GLEnumGroupGetPName::GL_MAX_FRAGMENT_INPUT_COMPONENTS => 1,
            ffi::GLEnumGroupGetPName::GL_MAX_FRAGMENT_UNIFORM_BLOCKS => 1,
            ffi::GLEnumGroupGetPName::GL_MAX_FRAGMENT_UNIFORM_COMPONENTS => 1,
            ffi::GLEnumGroupGetPName::GL_MAX_FRAGMENT_UNIFORM_VECTORS => 1,
            ffi::GLEnumGroupGetPName::GL_MAX_PROGRAM_TEXEL_OFFSET => 1,
            ffi::GLEnumGroupGetPName::GL_MAX_RENDERBUFFER_SIZE => 1,
            ffi::GLEnumGroupGetPName::GL_MAX_SERVER_WAIT_TIMEOUT => 1,
            ffi::GLEnumGroupGetPName::GL_MAX_TEXTURE_IMAGE_UNITS => 1,
            ffi::GLEnumGroupGetPName::GL_MAX_TEXTURE_LOD_BIAS => 1,
            ffi::GLEnumGroupGetPName::GL_MAX_TEXTURE_SIZE => 1,
            ffi::GLEnumGroupGetPName::GL_MAX_UNIFORM_BLOCK_SIZE => 1,
            ffi::GLEnumGroupGetPName::GL_MAX_UNIFORM_BUFFER_BINDINGS => 1,
            ffi::GLEnumGroupGetPName::GL_MAX_VARYING_COMPONENTS => 1,
            ffi::GLEnumGroupGetPName::GL_MAX_VARYING_VECTORS => 1,
            ffi::GLEnumGroupGetPName::GL_MAX_VERTEX_ATTRIBS => 1,
            ffi::GLEnumGroupGetPName::GL_MAX_VERTEX_OUTPUT_COMPONENTS => 1,
            ffi::GLEnumGroupGetPName::GL_MAX_VERTEX_TEXTURE_IMAGE_UNITS => 1,
            ffi::GLEnumGroupGetPName::GL_MAX_VERTEX_UNIFORM_BLOCKS => 1,
            ffi::GLEnumGroupGetPName::GL_MAX_VERTEX_UNIFORM_COMPONENTS => 1,
            ffi::GLEnumGroupGetPName::GL_MAX_VERTEX_UNIFORM_VECTORS => 1,
            ffi::GLEnumGroupGetPName::GL_MAX_VIEWPORT_DIMS => 2,
            ffi::GLEnumGroupGetPName::GL_MINOR_VERSION => 1,
            ffi::GLEnumGroupGetPName::GL_MIN_PROGRAM_TEXEL_OFFSET => 1,
            ffi::GLEnumGroupGetPName::GL_NUM_COMPRESSED_TEXTURE_FORMATS => 1,
            ffi::GLEnumGroupGetPName::GL_NUM_EXTENSIONS => 1,
            ffi::GLEnumGroupGetPName::GL_NUM_PROGRAM_BINARY_FORMATS => 1,
            ffi::GLEnumGroupGetPName::GL_NUM_SHADER_BINARY_FORMATS => 1,
            ffi::GLEnumGroupGetPName::GL_PACK_ALIGNMENT => 1,
            ffi::GLEnumGroupGetPName::GL_PACK_ROW_LENGTH => 1,
            ffi::GLEnumGroupGetPName::GL_PACK_ROW_LENGTH_NV => 1,
            ffi::GLEnumGroupGetPName::GL_PACK_SKIP_PIXELS => 1,
            ffi::GLEnumGroupGetPName::GL_PACK_SKIP_PIXELS_NV => 1,
            ffi::GLEnumGroupGetPName::GL_PACK_SKIP_ROWS => 1,
            ffi::GLEnumGroupGetPName::GL_PACK_SKIP_ROWS_NV => 1,
            ffi::GLEnumGroupGetPName::GL_PIXEL_PACK_BUFFER_BINDING => 1,
            ffi::GLEnumGroupGetPName::GL_PIXEL_UNPACK_BUFFER_BINDING => 1,
            ffi::GLEnumGroupGetPName::GL_POLYGON_OFFSET_FACTOR => 1,
            ffi::GLEnumGroupGetPName::GL_POLYGON_OFFSET_FILL => 1,
            ffi::GLEnumGroupGetPName::GL_POLYGON_OFFSET_UNITS => 1,
            ffi::GLEnumGroupGetPName::GL_PROGRAM_BINARY_FORMATS => {
                ffi::GL_NUM_PROGRAM_BINARY_FORMATS as usize
            }
            ffi::GLEnumGroupGetPName::GL_READ_BUFFER => 1,
            ffi::GLEnumGroupGetPName::GL_READ_FRAMEBUFFER_BINDING => 1,
            ffi::GLEnumGroupGetPName::GL_RED_BITS => 1,
            ffi::GLEnumGroupGetPName::GL_RENDERBUFFER_BINDING => 1,
            ffi::GLEnumGroupGetPName::GL_SAMPLER_BINDING => 1,
            ffi::GLEnumGroupGetPName::GL_SAMPLES => 1,
            ffi::GLEnumGroupGetPName::GL_SAMPLE_BUFFERS => 1,
            ffi::GLEnumGroupGetPName::GL_SAMPLE_COVERAGE_INVERT => 1,
            ffi::GLEnumGroupGetPName::GL_SAMPLE_COVERAGE_VALUE => 1,
            ffi::GLEnumGroupGetPName::GL_SCISSOR_BOX => 4,
            ffi::GLEnumGroupGetPName::GL_SCISSOR_TEST => 1,
            ffi::GLEnumGroupGetPName::GL_SHADER_BINARY_FORMATS => {
                ffi::GL_NUM_SHADER_BINARY_FORMATS as usize
            }
            ffi::GLEnumGroupGetPName::GL_SHADER_COMPILER => 1,
            ffi::GLEnumGroupGetPName::GL_SHADING_RATE_IMAGE_PALETTE_COUNT_NV => 1,
            ffi::GLEnumGroupGetPName::GL_SHADING_RATE_IMAGE_PER_PRIMITIVE_NV => 1,
            ffi::GLEnumGroupGetPName::GL_STENCIL_BACK_FAIL => 1,
            ffi::GLEnumGroupGetPName::GL_STENCIL_BACK_FUNC => 1,
            ffi::GLEnumGroupGetPName::GL_STENCIL_BACK_PASS_DEPTH_FAIL => 1,
            ffi::GLEnumGroupGetPName::GL_STENCIL_BACK_PASS_DEPTH_PASS => 1,
            ffi::GLEnumGroupGetPName::GL_STENCIL_BACK_REF => 1,
            ffi::GLEnumGroupGetPName::GL_STENCIL_BACK_VALUE_MASK => 1,
            ffi::GLEnumGroupGetPName::GL_STENCIL_BACK_WRITEMASK => 1,
            ffi::GLEnumGroupGetPName::GL_STENCIL_BITS => 1,
            ffi::GLEnumGroupGetPName::GL_STENCIL_CLEAR_VALUE => 1,
            ffi::GLEnumGroupGetPName::GL_STENCIL_FAIL => 1,
            ffi::GLEnumGroupGetPName::GL_STENCIL_FUNC => 1,
            ffi::GLEnumGroupGetPName::GL_STENCIL_PASS_DEPTH_FAIL => 1,
            ffi::GLEnumGroupGetPName::GL_STENCIL_PASS_DEPTH_PASS => 1,
            ffi::GLEnumGroupGetPName::GL_STENCIL_REF => 1,
            ffi::GLEnumGroupGetPName::GL_STENCIL_TEST => 1,
            ffi::GLEnumGroupGetPName::GL_STENCIL_VALUE_MASK => 1,
            ffi::GLEnumGroupGetPName::GL_STENCIL_WRITEMASK => 1,
            ffi::GLEnumGroupGetPName::GL_SUBPIXEL_BITS => 1,
            ffi::GLEnumGroupGetPName::GL_TEXTURE_2D => 1,
            ffi::GLEnumGroupGetPName::GL_TEXTURE_BINDING_2D => 1,
            ffi::GLEnumGroupGetPName::GL_TEXTURE_BINDING_2D_ARRAY => 1,
            ffi::GLEnumGroupGetPName::GL_TEXTURE_BINDING_3D => 1,
            ffi::GLEnumGroupGetPName::GL_TEXTURE_BINDING_CUBE_MAP => 1,
            ffi::GLEnumGroupGetPName::GL_TRANSFORM_FEEDBACK_BUFFER_BINDING => 1,
            ffi::GLEnumGroupGetPName::GL_TRANSFORM_FEEDBACK_BUFFER_SIZE => 1,
            ffi::GLEnumGroupGetPName::GL_TRANSFORM_FEEDBACK_BUFFER_START => 1,
            ffi::GLEnumGroupGetPName::GL_UNIFORM_BUFFER_BINDING => 1,
            ffi::GLEnumGroupGetPName::GL_UNIFORM_BUFFER_OFFSET_ALIGNMENT => 1,
            ffi::GLEnumGroupGetPName::GL_UNIFORM_BUFFER_SIZE => 1,
            ffi::GLEnumGroupGetPName::GL_UNIFORM_BUFFER_START => 1,
            ffi::GLEnumGroupGetPName::GL_UNPACK_ALIGNMENT => 1,
            ffi::GLEnumGroupGetPName::GL_UNPACK_IMAGE_HEIGHT => 1,
            ffi::GLEnumGroupGetPName::GL_UNPACK_ROW_LENGTH => 1,
            ffi::GLEnumGroupGetPName::GL_UNPACK_SKIP_IMAGES => 1,
            ffi::GLEnumGroupGetPName::GL_UNPACK_SKIP_PIXELS => 1,
            ffi::GLEnumGroupGetPName::GL_UNPACK_SKIP_ROWS => 1,
            ffi::GLEnumGroupGetPName::GL_VERTEX_ARRAY_BINDING => 1,
            ffi::GLEnumGroupGetPName::GL_VIEWPORT => 4,
        },
    }
}
#[allow(non_snake_case)]
pub(crate) fn glGetFloatv_data_compsize(
    _gl: &mut GL,
    pname: Option<ffi::GLEnumGroupGetPName>,
) -> usize {
    glGetBooleanv_data_compsize(_gl, pname)
}
#[allow(non_snake_case)]
pub(crate) fn glGetIntegerv_data_compsize(
    _gl: &mut GL,
    pname: Option<ffi::GLEnumGroupGetPName>,
) -> usize {
    glGetBooleanv_data_compsize(_gl, pname)
}

#[allow(non_snake_case)]
pub(crate) fn glGetInteger64v_data_compsize(
    _gl: &mut GL,
    pname: Option<ffi::GLEnumGroupGetPName>,
) -> usize {
    glGetBooleanv_data_compsize(_gl, pname)
}

#[allow(non_snake_case)]
pub(crate) fn glGetInteger64i_v_data_compsize(
    _gl: &mut GL,
    _pname: Option<ffi::GLEnumGroupGetPName>,
) -> usize {
    1 // indexed
}
#[allow(non_snake_case)]
pub(crate) fn glGetIntegeri_v_data_compsize(
    _gl: &mut GL,
    _pname: Option<ffi::GLEnumGroupGetPName>,
) -> usize {
    1 // indexed
}

// GLEnumGroupBufferPNameARB

#[allow(non_snake_case)]
pub(crate) fn glGetBufferParameteri64v_params_compsize(
    _gl: &mut GL,
    pname: Option<ffi::GLEnumGroupBufferPNameARB>,
) -> usize {
    match pname {
        None => 1,
        Some(pname) => match pname {
            ffi::GLEnumGroupBufferPNameARB::GL_BUFFER_ACCESS_FLAGS => 1,
            ffi::GLEnumGroupBufferPNameARB::GL_BUFFER_MAPPED => 1,
            ffi::GLEnumGroupBufferPNameARB::GL_BUFFER_MAP_LENGTH => 1,
            ffi::GLEnumGroupBufferPNameARB::GL_BUFFER_MAP_OFFSET => 1,
            ffi::GLEnumGroupBufferPNameARB::GL_BUFFER_SIZE => 1,
            ffi::GLEnumGroupBufferPNameARB::GL_BUFFER_USAGE => 1,
        },
    }
}

#[allow(non_snake_case)]
pub(crate) fn glGetBufferParameteriv_params_compsize(
    _gl: &mut GL,
    _pname: Option<ffi::GLEnumGroupBufferPNameARB>,
) -> usize {
    glGetBufferParameteri64v_params_compsize(_gl, _pname)
}

// GLEnumGroupFramebufferAttachmentParameterName

#[allow(non_snake_case)]
pub(crate) fn glGetFramebufferAttachmentParameteriv_params_compsize(
    _gl: &mut GL,
    pname: Option<ffi::GLEnumGroupFramebufferAttachmentParameterName>,
) -> usize {
    match pname {
        None => 1,
        Some(pname) => match pname {
            ffi::GLEnumGroupFramebufferAttachmentParameterName::GL_FRAMEBUFFER_ATTACHMENT_ALPHA_SIZE =>1,
            ffi::GLEnumGroupFramebufferAttachmentParameterName::GL_FRAMEBUFFER_ATTACHMENT_BLUE_SIZE =>1,
            ffi::GLEnumGroupFramebufferAttachmentParameterName::GL_FRAMEBUFFER_ATTACHMENT_COLOR_ENCODING =>1,
            ffi::GLEnumGroupFramebufferAttachmentParameterName::GL_FRAMEBUFFER_ATTACHMENT_COLOR_ENCODING_EXT =>1,
            ffi::GLEnumGroupFramebufferAttachmentParameterName::GL_FRAMEBUFFER_ATTACHMENT_COMPONENT_TYPE =>1,ffi::GLEnumGroupFramebufferAttachmentParameterName::GL_FRAMEBUFFER_ATTACHMENT_COMPONENT_TYPE_EXT =>1,ffi::GLEnumGroupFramebufferAttachmentParameterName::GL_FRAMEBUFFER_ATTACHMENT_DEPTH_SIZE =>1,ffi::GLEnumGroupFramebufferAttachmentParameterName::GL_FRAMEBUFFER_ATTACHMENT_GREEN_SIZE =>1,ffi::GLEnumGroupFramebufferAttachmentParameterName::GL_FRAMEBUFFER_ATTACHMENT_OBJECT_NAME =>1,ffi::GLEnumGroupFramebufferAttachmentParameterName::GL_FRAMEBUFFER_ATTACHMENT_OBJECT_TYPE =>1,ffi::GLEnumGroupFramebufferAttachmentParameterName::GL_FRAMEBUFFER_ATTACHMENT_RED_SIZE =>1,ffi::GLEnumGroupFramebufferAttachmentParameterName::GL_FRAMEBUFFER_ATTACHMENT_STENCIL_SIZE =>1,ffi::GLEnumGroupFramebufferAttachmentParameterName::GL_FRAMEBUFFER_ATTACHMENT_TEXTURE_CUBE_MAP_FACE =>1,ffi::GLEnumGroupFramebufferAttachmentParameterName::GL_FRAMEBUFFER_ATTACHMENT_TEXTURE_LAYER =>1,ffi::GLEnumGroupFramebufferAttachmentParameterName::GL_FRAMEBUFFER_ATTACHMENT_TEXTURE_LEVEL =>1,
        },
    }
}

// GLEnumGroupProgramPropertyARB

#[allow(non_snake_case)]
pub(crate) fn glGetProgramiv_params_compsize(
    _gl: &mut GL,
    pname: Option<ffi::GLEnumGroupProgramPropertyARB>,
) -> usize {
    match pname {
        None => 1,
        Some(pname) => match pname {
            ffi::GLEnumGroupProgramPropertyARB::GL_ACTIVE_ATTRIBUTES => 1,
            ffi::GLEnumGroupProgramPropertyARB::GL_ACTIVE_ATTRIBUTE_MAX_LENGTH => 1,
            ffi::GLEnumGroupProgramPropertyARB::GL_ACTIVE_UNIFORMS => 1,
            ffi::GLEnumGroupProgramPropertyARB::GL_ACTIVE_UNIFORM_BLOCKS => 1, // TODO check
            ffi::GLEnumGroupProgramPropertyARB::GL_ACTIVE_UNIFORM_BLOCK_MAX_NAME_LENGTH => 1, // TODO check
            ffi::GLEnumGroupProgramPropertyARB::GL_ACTIVE_UNIFORM_MAX_LENGTH => 1,
            ffi::GLEnumGroupProgramPropertyARB::GL_ATTACHED_SHADERS => 1,
            ffi::GLEnumGroupProgramPropertyARB::GL_DELETE_STATUS => 1,
            ffi::GLEnumGroupProgramPropertyARB::GL_INFO_LOG_LENGTH => 1,
            ffi::GLEnumGroupProgramPropertyARB::GL_LINK_STATUS => 1,
            ffi::GLEnumGroupProgramPropertyARB::GL_PROGRAM_BINARY_LENGTH => 1, // TODO check
            ffi::GLEnumGroupProgramPropertyARB::GL_TRANSFORM_FEEDBACK_BUFFER_MODE => 1, // TODO check
            ffi::GLEnumGroupProgramPropertyARB::GL_TRANSFORM_FEEDBACK_VARYINGS => 1, // TODO check
            ffi::GLEnumGroupProgramPropertyARB::GL_TRANSFORM_FEEDBACK_VARYING_MAX_LENGTH => 1, // TODO check
            ffi::GLEnumGroupProgramPropertyARB::GL_VALIDATE_STATUS => 1,
        },
    }
}

//GLEnumGroupQueryObjectParameterName

#[allow(non_snake_case)]
pub(crate) fn glGetQueryObjectuiv_params_compsize(
    _gl: &mut GL,
    pname: Option<ffi::GLEnumGroupQueryObjectParameterName>,
) -> usize {
    match pname {
        None => 1,
        Some(pname) => match pname {
            ffi::GLEnumGroupQueryObjectParameterName::GL_QUERY_RESULT => 1,
            ffi::GLEnumGroupQueryObjectParameterName::GL_QUERY_RESULT_AVAILABLE => 1,
        },
    }
}

// GLEnumGroupQueryParameterName

#[allow(non_snake_case)]
pub(crate) fn glGetQueryiv_params_compsize(
    _gl: &mut GL,
    pname: Option<ffi::GLEnumGroupQueryParameterName>,
) -> usize {
    match pname {
        None => 1,
        Some(pname) => match pname {
            ffi::GLEnumGroupQueryParameterName::GL_CURRENT_QUERY => 1,
        },
    }
}

// GLEnumGroupRenderbufferParameterName

#[allow(non_snake_case)]
pub(crate) fn glGetRenderbufferParameteriv_params_compsize(
    _gl: &mut GL,
    pname: Option<ffi::GLEnumGroupRenderbufferParameterName>,
) -> usize {
    match pname {
        None => 1,
        Some(pname) => match pname {
            ffi::GLEnumGroupRenderbufferParameterName::GL_RENDERBUFFER_ALPHA_SIZE => 1,
            ffi::GLEnumGroupRenderbufferParameterName::GL_RENDERBUFFER_BLUE_SIZE => 1,
            ffi::GLEnumGroupRenderbufferParameterName::GL_RENDERBUFFER_DEPTH_SIZE => 1,
            ffi::GLEnumGroupRenderbufferParameterName::GL_RENDERBUFFER_GREEN_SIZE => 1,
            ffi::GLEnumGroupRenderbufferParameterName::GL_RENDERBUFFER_HEIGHT => 1,
            ffi::GLEnumGroupRenderbufferParameterName::GL_RENDERBUFFER_INTERNAL_FORMAT => 1,
            ffi::GLEnumGroupRenderbufferParameterName::GL_RENDERBUFFER_RED_SIZE => 1,
            ffi::GLEnumGroupRenderbufferParameterName::GL_RENDERBUFFER_SAMPLES => 1,
            ffi::GLEnumGroupRenderbufferParameterName::GL_RENDERBUFFER_STENCIL_SIZE => 1,
            ffi::GLEnumGroupRenderbufferParameterName::GL_RENDERBUFFER_WIDTH => 1,
        },
    }
}

// GLEnumGroupSamplerParameterF

#[allow(non_snake_case)]
pub(crate) fn glGetSamplerParameterfv_params_compsize(
    _gl: &mut GL,
    pname: Option<ffi::GLEnumGroupSamplerParameterF>,
) -> usize {
    match pname {
        None => 1,
        Some(pname) => match pname {
            ffi::GLEnumGroupSamplerParameterF::GL_TEXTURE_MAX_LOD => 1, // TODO check
            ffi::GLEnumGroupSamplerParameterF::GL_TEXTURE_MIN_LOD => 1, // TODO check
            ffi::GLEnumGroupSamplerParameterF::GL_TEXTURE_UNNORMALIZED_COORDINATES_ARM => 1, // TODO check
        },
    }
}

// GLEnumGroupSamplerParameterI

#[allow(non_snake_case)]
pub(crate) fn glGetSamplerParameteriv_params_compsize(
    _gl: &mut GL,
    pname: Option<ffi::GLEnumGroupSamplerParameterI>,
) -> usize {
    match pname {
        None => 1,
        Some(pname) => match pname {
            ffi::GLEnumGroupSamplerParameterI::GL_TEXTURE_COMPARE_FUNC => 1,
            ffi::GLEnumGroupSamplerParameterI::GL_TEXTURE_COMPARE_MODE => 1,
            ffi::GLEnumGroupSamplerParameterI::GL_TEXTURE_MAG_FILTER => 1,
            ffi::GLEnumGroupSamplerParameterI::GL_TEXTURE_MIN_FILTER => 1,
            ffi::GLEnumGroupSamplerParameterI::GL_TEXTURE_UNNORMALIZED_COORDINATES_ARM => 1, // TODO check
            ffi::GLEnumGroupSamplerParameterI::GL_TEXTURE_WRAP_R => 1,
            ffi::GLEnumGroupSamplerParameterI::GL_TEXTURE_WRAP_S => 1,
            ffi::GLEnumGroupSamplerParameterI::GL_TEXTURE_WRAP_T => 1,
        },
    }
}

#[allow(non_snake_case)]
pub(crate) fn glSamplerParameteriv_param_compsize(
    _gl: &mut GL,
    pname: Option<ffi::GLEnumGroupSamplerParameterI>,
) -> usize {
    glGetSamplerParameteriv_params_compsize(_gl, pname)
}

// GLEnumGroupShaderParameterName

#[allow(non_snake_case)]
pub(crate) fn glGetShaderiv_params_compsize(
    _gl: &mut GL,
    pname: Option<ffi::GLEnumGroupShaderParameterName>,
) -> usize {
    match pname {
        None => 1,
        Some(pname) => match pname {
            ffi::GLEnumGroupShaderParameterName::GL_COMPILE_STATUS => 1,
            ffi::GLEnumGroupShaderParameterName::GL_DELETE_STATUS => 1,
            ffi::GLEnumGroupShaderParameterName::GL_INFO_LOG_LENGTH => 1,
            ffi::GLEnumGroupShaderParameterName::GL_SHADER_SOURCE_LENGTH => 1,
            ffi::GLEnumGroupShaderParameterName::GL_SHADER_TYPE => 1,
        },
    }
}

// GLEnumGroupGetTextureParameter

#[allow(non_snake_case)]
pub(crate) fn glGetTexParameterfv_params_compsize(
    _gl: &mut GL,
    pname: Option<ffi::GLEnumGroupGetTextureParameter>,
) -> usize {
    match pname {
        None => 1,
        Some(pname) => match pname {
            ffi::GLEnumGroupGetTextureParameter::GL_SURFACE_COMPRESSION_EXT => 1, // TODO check
            ffi::GLEnumGroupGetTextureParameter::GL_TEXTURE_BORDER_COLOR_NV => 4, // TODO check
            ffi::GLEnumGroupGetTextureParameter::GL_TEXTURE_CBCR_DEGAMMA_QCOM => 1, // TODO check
            ffi::GLEnumGroupGetTextureParameter::GL_TEXTURE_MAG_FILTER => 1,
            ffi::GLEnumGroupGetTextureParameter::GL_TEXTURE_MIN_FILTER => 1,
            ffi::GLEnumGroupGetTextureParameter::GL_TEXTURE_UNNORMALIZED_COORDINATES_ARM => 1, // TODO check
            ffi::GLEnumGroupGetTextureParameter::GL_TEXTURE_WRAP_S => 1,
            ffi::GLEnumGroupGetTextureParameter::GL_TEXTURE_WRAP_T => 1,
            ffi::GLEnumGroupGetTextureParameter::GL_TEXTURE_Y_DEGAMMA_QCOM => 1, // TODO check
        },
    }
}

#[allow(non_snake_case)]
pub(crate) fn glGetTexParameteriv_params_compsize(
    _gl: &mut GL,
    pname: Option<ffi::GLEnumGroupGetTextureParameter>,
) -> usize {
    glGetTexParameterfv_params_compsize(_gl, pname)
}

// GLEnumGroupPixelFormat
// GLEnumGroupPixelType

fn pixel_type_width(
    format: Option<ffi::GLEnumGroupPixelFormat>,
    ty: Option<ffi::GLEnumGroupPixelType>,
) -> usize {
    gl_type_width(ty)
        * std::cmp::max(
            gl_format_component_num(format) / gl_type_component_num(ty),
            1,
        )
}

fn gl_type_width(ty: Option<ffi::GLEnumGroupPixelType>) -> usize {
    match ty {
        None => 4,
        Some(ty) => match ty {
            ffi::GLEnumGroupPixelType::GL_BYTE => 1,
            ffi::GLEnumGroupPixelType::GL_FLOAT => 4,
            ffi::GLEnumGroupPixelType::GL_FLOAT_32_UNSIGNED_INT_24_8_REV => 8,
            ffi::GLEnumGroupPixelType::GL_HALF_FLOAT => 2,
            ffi::GLEnumGroupPixelType::GL_INT => 4,
            ffi::GLEnumGroupPixelType::GL_SHORT => 2,
            ffi::GLEnumGroupPixelType::GL_UNSIGNED_BYTE => 1,
            ffi::GLEnumGroupPixelType::GL_UNSIGNED_INT => 4,
            ffi::GLEnumGroupPixelType::GL_UNSIGNED_INT_10F_11F_11F_REV => 4,
            ffi::GLEnumGroupPixelType::GL_UNSIGNED_INT_10F_11F_11F_REV_APPLE => 4,
            ffi::GLEnumGroupPixelType::GL_UNSIGNED_INT_24_8 => 4,
            ffi::GLEnumGroupPixelType::GL_UNSIGNED_INT_24_8_OES => 4,
            ffi::GLEnumGroupPixelType::GL_UNSIGNED_INT_2_10_10_10_REV => 4,
            ffi::GLEnumGroupPixelType::GL_UNSIGNED_INT_2_10_10_10_REV_EXT => 4,
            ffi::GLEnumGroupPixelType::GL_UNSIGNED_INT_5_9_9_9_REV => 4,
            ffi::GLEnumGroupPixelType::GL_UNSIGNED_INT_5_9_9_9_REV_APPLE => 4,
            ffi::GLEnumGroupPixelType::GL_UNSIGNED_SHORT => 2,
            ffi::GLEnumGroupPixelType::GL_UNSIGNED_SHORT_1_5_5_5_REV_EXT => 2,
            ffi::GLEnumGroupPixelType::GL_UNSIGNED_SHORT_4_4_4_4 => 2,
            ffi::GLEnumGroupPixelType::GL_UNSIGNED_SHORT_4_4_4_4_REV_EXT => 2,
            ffi::GLEnumGroupPixelType::GL_UNSIGNED_SHORT_4_4_4_4_REV_IMG => 2,
            ffi::GLEnumGroupPixelType::GL_UNSIGNED_SHORT_5_5_5_1 => 2,
            ffi::GLEnumGroupPixelType::GL_UNSIGNED_SHORT_5_6_5 => 2,
        },
    }
}

fn gl_type_component_num(ty: Option<ffi::GLEnumGroupPixelType>) -> usize {
    match ty {
        None => 4,
        Some(ty) => match ty {
            ffi::GLEnumGroupPixelType::GL_BYTE => 1,
            ffi::GLEnumGroupPixelType::GL_FLOAT => 1,
            ffi::GLEnumGroupPixelType::GL_FLOAT_32_UNSIGNED_INT_24_8_REV => 3,
            ffi::GLEnumGroupPixelType::GL_HALF_FLOAT => 1,
            ffi::GLEnumGroupPixelType::GL_INT => 1,
            ffi::GLEnumGroupPixelType::GL_SHORT => 1,
            ffi::GLEnumGroupPixelType::GL_UNSIGNED_BYTE => 1,
            ffi::GLEnumGroupPixelType::GL_UNSIGNED_INT => 1,
            ffi::GLEnumGroupPixelType::GL_UNSIGNED_INT_10F_11F_11F_REV => 3,
            ffi::GLEnumGroupPixelType::GL_UNSIGNED_INT_10F_11F_11F_REV_APPLE => 3,
            ffi::GLEnumGroupPixelType::GL_UNSIGNED_INT_24_8 => 2,
            ffi::GLEnumGroupPixelType::GL_UNSIGNED_INT_24_8_OES => 2,
            ffi::GLEnumGroupPixelType::GL_UNSIGNED_INT_2_10_10_10_REV => 4,
            ffi::GLEnumGroupPixelType::GL_UNSIGNED_INT_2_10_10_10_REV_EXT => 4,
            ffi::GLEnumGroupPixelType::GL_UNSIGNED_INT_5_9_9_9_REV => 4,
            ffi::GLEnumGroupPixelType::GL_UNSIGNED_INT_5_9_9_9_REV_APPLE => 4,
            ffi::GLEnumGroupPixelType::GL_UNSIGNED_SHORT => 1,
            ffi::GLEnumGroupPixelType::GL_UNSIGNED_SHORT_1_5_5_5_REV_EXT => 4,
            ffi::GLEnumGroupPixelType::GL_UNSIGNED_SHORT_4_4_4_4 => 4,
            ffi::GLEnumGroupPixelType::GL_UNSIGNED_SHORT_4_4_4_4_REV_EXT => 4,
            ffi::GLEnumGroupPixelType::GL_UNSIGNED_SHORT_4_4_4_4_REV_IMG => 4,
            ffi::GLEnumGroupPixelType::GL_UNSIGNED_SHORT_5_5_5_1 => 4,
            ffi::GLEnumGroupPixelType::GL_UNSIGNED_SHORT_5_6_5 => 3,
        },
    }
}

fn gl_format_component_num(format: Option<ffi::GLEnumGroupPixelFormat>) -> usize {
    match format {
        None => 4,
        Some(format) => match format {
            ffi::GLEnumGroupPixelFormat::GL_ALPHA => 1,
            ffi::GLEnumGroupPixelFormat::GL_BGRA_EXT => 4,
            ffi::GLEnumGroupPixelFormat::GL_BGRA_IMG => 4,
            ffi::GLEnumGroupPixelFormat::GL_BGR_EXT => 3,
            ffi::GLEnumGroupPixelFormat::GL_BLUE => 1,
            ffi::GLEnumGroupPixelFormat::GL_DEPTH_COMPONENT => 1,
            ffi::GLEnumGroupPixelFormat::GL_DEPTH_STENCIL => 1,
            ffi::GLEnumGroupPixelFormat::GL_GREEN => 1,
            ffi::GLEnumGroupPixelFormat::GL_LUMINANCE => 1,
            ffi::GLEnumGroupPixelFormat::GL_LUMINANCE_ALPHA => 2,
            ffi::GLEnumGroupPixelFormat::GL_RED => 1,
            ffi::GLEnumGroupPixelFormat::GL_RED_EXT => 1,
            ffi::GLEnumGroupPixelFormat::GL_RED_INTEGER => 1,
            ffi::GLEnumGroupPixelFormat::GL_RG => 2,
            ffi::GLEnumGroupPixelFormat::GL_RGB => 3,
            ffi::GLEnumGroupPixelFormat::GL_RGBA => 4,
            ffi::GLEnumGroupPixelFormat::GL_RGBA_INTEGER => 4,
            ffi::GLEnumGroupPixelFormat::GL_RGB_INTEGER => 3,
            ffi::GLEnumGroupPixelFormat::GL_RG_INTEGER => 2,
            ffi::GLEnumGroupPixelFormat::GL_UNSIGNED_INT => 1,
            ffi::GLEnumGroupPixelFormat::GL_UNSIGNED_SHORT => 1,
        },
    }
}

#[allow(non_snake_case)]
pub(crate) fn glReadPixels_pixels_compsize(
    _gl: &mut GL,
    format: Option<ffi::GLEnumGroupPixelFormat>,
    ty: Option<ffi::GLEnumGroupPixelType>,
    width: i32,
    height: i32,
) -> usize {
    return (width * height) as usize * pixel_type_width(format, ty);
}
#[allow(non_snake_case)]
pub(crate) fn glTexImage3D_pixels_compsize(
    _gl: &mut GL,
    format: Option<ffi::GLEnumGroupPixelFormat>,
    ty: Option<ffi::GLEnumGroupPixelType>,
    width: i32,
    height: i32,
    depth: i32,
) -> usize {
    return (width * height * depth) as usize * pixel_type_width(format, ty);
}
#[allow(non_snake_case)]
pub(crate) fn glTexSubImage2D_pixels_compsize(
    _gl: &mut GL,
    format: Option<ffi::GLEnumGroupPixelFormat>,
    ty: Option<ffi::GLEnumGroupPixelType>,
    width: i32,
    height: i32,
) -> usize {
    return (width * height) as usize * pixel_type_width(format, ty);
}
#[allow(non_snake_case)]
pub(crate) fn glTexSubImage3D_pixels_compsize(
    _gl: &mut GL,
    format: Option<ffi::GLEnumGroupPixelFormat>,
    ty: Option<ffi::GLEnumGroupPixelType>,
    width: i32,
    height: i32,
    depth: i32,
) -> usize {
    return (width * height * depth) as usize * pixel_type_width(format, ty);
}

#[allow(non_snake_case)]
pub(crate) fn glTexImage2D_pixels_compsize(
    _gl: &mut GL,
    format: Option<ffi::GLEnumGroupPixelFormat>,
    ty: Option<ffi::GLEnumGroupPixelType>,
    width: i32,
    height: i32,
) -> usize {
    return (width * height) as usize * pixel_type_width(format, ty);
}

// GLEnumGroupSamplerParameterF

#[allow(non_snake_case)]
pub(crate) fn glSamplerParameterfv_param_compsize(
    _gl: &mut GL,
    pname: Option<ffi::GLEnumGroupSamplerParameterF>,
) -> usize {
    match pname {
        None => 1,
        Some(pname) => match pname {
            ffi::GLEnumGroupSamplerParameterF::GL_TEXTURE_MAX_LOD => 1,
            ffi::GLEnumGroupSamplerParameterF::GL_TEXTURE_MIN_LOD => 1,
            ffi::GLEnumGroupSamplerParameterF::GL_TEXTURE_UNNORMALIZED_COORDINATES_ARM => 1, // TODO check
        },
    }
}

// GLEnumGroupTextureParameterName

#[allow(non_snake_case)]
pub(crate) fn glTexParameterfv_params_compsize(
    _gl: &mut GL,
    pname: Option<ffi::GLEnumGroupTextureParameterName>,
) -> usize {
    match pname {
        None => 1,
        Some(pname) => match pname {
            ffi::GLEnumGroupTextureParameterName::GL_TEXTURE_BASE_LEVEL => 1,
            ffi::GLEnumGroupTextureParameterName::GL_TEXTURE_BORDER_COLOR_NV => 4, // TODO check
            ffi::GLEnumGroupTextureParameterName::GL_TEXTURE_CBCR_DEGAMMA_QCOM => 1, // TODO check
            ffi::GLEnumGroupTextureParameterName::GL_TEXTURE_COMPARE_FUNC => 1,
            ffi::GLEnumGroupTextureParameterName::GL_TEXTURE_COMPARE_MODE => 1,
            ffi::GLEnumGroupTextureParameterName::GL_TEXTURE_FOVEATED_CUTOFF_DENSITY_QCOM => 1, // TODO check
            ffi::GLEnumGroupTextureParameterName::GL_TEXTURE_MAG_FILTER => 1,
            ffi::GLEnumGroupTextureParameterName::GL_TEXTURE_MAX_LEVEL => 1,
            ffi::GLEnumGroupTextureParameterName::GL_TEXTURE_MAX_LOD => 1,
            ffi::GLEnumGroupTextureParameterName::GL_TEXTURE_MIN_FILTER => 1,
            ffi::GLEnumGroupTextureParameterName::GL_TEXTURE_MIN_LOD => 1,
            ffi::GLEnumGroupTextureParameterName::GL_TEXTURE_SWIZZLE_A => 1,
            ffi::GLEnumGroupTextureParameterName::GL_TEXTURE_SWIZZLE_B => 1,
            ffi::GLEnumGroupTextureParameterName::GL_TEXTURE_SWIZZLE_G => 1,
            ffi::GLEnumGroupTextureParameterName::GL_TEXTURE_SWIZZLE_R => 1,
            ffi::GLEnumGroupTextureParameterName::GL_TEXTURE_UNNORMALIZED_COORDINATES_ARM => 1, // TODO check
            ffi::GLEnumGroupTextureParameterName::GL_TEXTURE_WRAP_R => 1,
            ffi::GLEnumGroupTextureParameterName::GL_TEXTURE_WRAP_S => 1,
            ffi::GLEnumGroupTextureParameterName::GL_TEXTURE_WRAP_T => 1,
            ffi::GLEnumGroupTextureParameterName::GL_TEXTURE_Y_DEGAMMA_QCOM => 1, // TODO check
        },
    }
}
#[allow(non_snake_case)]
pub(crate) fn glTexParameteriv_params_compsize(
    _gl: &mut GL,
    pname: Option<ffi::GLEnumGroupTextureParameterName>,
) -> usize {
    glTexParameterfv_params_compsize(_gl, pname)
}

// GLEnumGroupDrawElementsType

#[allow(non_snake_case)]
pub(crate) fn glDrawElements_indices_compsize(
    _gl: &mut GL,
    count: i32,
    _type: Option<ffi::GLEnumGroupDrawElementsType>,
) -> usize {
    let type_width = match _type {
        None => 1,
        Some(_type) => match _type {
            ffi::GLEnumGroupDrawElementsType::GL_UNSIGNED_BYTE => 1,
            ffi::GLEnumGroupDrawElementsType::GL_UNSIGNED_INT => 4,
            ffi::GLEnumGroupDrawElementsType::GL_UNSIGNED_SHORT => 2,
        },
    };
    type_width * count as usize
}

// WIP

#[allow(non_snake_case)]
pub(crate) fn glGetUniformfv_params_compsize(_gl: &mut GL, _program: u32, _location: i32) -> usize {
    // TODO somehow check, cz it is unsafe
    // try glGetActiveUniform
    4
}

#[allow(non_snake_case)]
pub(crate) fn glGetUniformiv_params_compsize(_gl: &mut GL, _program: u32, _location: i32) -> usize {
    // TODO somehow check, cz it is unsafe
    // try glGetActiveUniform
    4
}
#[allow(non_snake_case)]
pub(crate) fn glGetUniformuiv_params_compsize(
    _gl: &mut GL,
    _program: u32,
    _location: i32,
) -> usize {
    // TODO somehow check, cz it is unsafe
    // try glGetActiveUniform
    4
}
