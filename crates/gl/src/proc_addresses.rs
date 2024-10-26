#![allow(non_snake_case)]
#[rustfmt::skip]
pub struct ProcAddresses {
    pub glActiveTexture: unsafe extern "stdcall" fn(texture: std::os::raw::c_uint) -> (),
    pub glAttachShader: unsafe extern "stdcall" fn(program: std::os::raw::c_uint,shader: std::os::raw::c_uint) -> (),
    pub glBeginQuery: unsafe extern "stdcall" fn(target: std::os::raw::c_uint,id: std::os::raw::c_uint) -> (),
    pub glBeginTransformFeedback: unsafe extern "stdcall" fn(primitiveMode: std::os::raw::c_uint) -> (),
    pub glBindAttribLocation: unsafe extern "stdcall" fn(program: std::os::raw::c_uint,index: std::os::raw::c_uint,name: *const i8) -> (),
    pub glBindBuffer: unsafe extern "stdcall" fn(target: std::os::raw::c_uint,buffer: std::os::raw::c_uint) -> (),
    pub glBindBufferBase: unsafe extern "stdcall" fn(target: std::os::raw::c_uint,index: std::os::raw::c_uint,buffer: std::os::raw::c_uint) -> (),
    pub glBindBufferRange: unsafe extern "stdcall" fn(target: std::os::raw::c_uint,index: std::os::raw::c_uint,buffer: std::os::raw::c_uint,offset: isize,size: isize) -> (),
    pub glBindFramebuffer: unsafe extern "stdcall" fn(target: std::os::raw::c_uint,framebuffer: std::os::raw::c_uint) -> (),
    pub glBindRenderbuffer: unsafe extern "stdcall" fn(target: std::os::raw::c_uint,renderbuffer: std::os::raw::c_uint) -> (),
    pub glBindSampler: unsafe extern "stdcall" fn(unit: std::os::raw::c_uint,sampler: std::os::raw::c_uint) -> (),
    pub glBindTexture: unsafe extern "stdcall" fn(target: std::os::raw::c_uint,texture: std::os::raw::c_uint) -> (),
    pub glBindTransformFeedback: unsafe extern "stdcall" fn(target: std::os::raw::c_uint,id: std::os::raw::c_uint) -> (),
    pub glBindVertexArray: unsafe extern "stdcall" fn(array: std::os::raw::c_uint) -> (),
    pub glBindVertexArrayOES: unsafe extern "stdcall" fn(array: std::os::raw::c_uint) -> (),
    pub glBlendColor: unsafe extern "stdcall" fn(red: f32,green: f32,blue: f32,alpha: f32) -> (),
    pub glBlendEquation: unsafe extern "stdcall" fn(mode: std::os::raw::c_uint) -> (),
    pub glBlendEquationSeparate: unsafe extern "stdcall" fn(modeRGB: std::os::raw::c_uint,modeAlpha: std::os::raw::c_uint) -> (),
    pub glBlendFunc: unsafe extern "stdcall" fn(sfactor: std::os::raw::c_uint,dfactor: std::os::raw::c_uint) -> (),
    pub glBlendFuncSeparate: unsafe extern "stdcall" fn(sfactorRGB: std::os::raw::c_uint,dfactorRGB: std::os::raw::c_uint,sfactorAlpha: std::os::raw::c_uint,dfactorAlpha: std::os::raw::c_uint) -> (),
    pub glBlitFramebuffer: unsafe extern "stdcall" fn(srcX0: std::os::raw::c_int,srcY0: std::os::raw::c_int,srcX1: std::os::raw::c_int,srcY1: std::os::raw::c_int,dstX0: std::os::raw::c_int,dstY0: std::os::raw::c_int,dstX1: std::os::raw::c_int,dstY1: std::os::raw::c_int,mask: std::os::raw::c_uint,filter: std::os::raw::c_uint) -> (),
    pub glBufferData: unsafe extern "stdcall" fn(target: std::os::raw::c_uint,size: isize,data: *const (),usage: std::os::raw::c_uint) -> (),
    pub glBufferSubData: unsafe extern "stdcall" fn(target: std::os::raw::c_uint,offset: isize,size: isize,data: *const ()) -> (),
    pub glCheckFramebufferStatus: unsafe extern "stdcall" fn(target: std::os::raw::c_uint) -> std::os::raw::c_uint,
    pub glClear: unsafe extern "stdcall" fn(mask: std::os::raw::c_uint) -> (),
    pub glClearBufferfi: unsafe extern "stdcall" fn(buffer: std::os::raw::c_uint,drawbuffer: std::os::raw::c_int,depth: f32,stencil: std::os::raw::c_int) -> (),
    pub glClearBufferfv: unsafe extern "stdcall" fn(buffer: std::os::raw::c_uint,drawbuffer: std::os::raw::c_int,value: *const f32) -> (),
    pub glClearBufferiv: unsafe extern "stdcall" fn(buffer: std::os::raw::c_uint,drawbuffer: std::os::raw::c_int,value: *const std::os::raw::c_int) -> (),
    pub glClearBufferuiv: unsafe extern "stdcall" fn(buffer: std::os::raw::c_uint,drawbuffer: std::os::raw::c_int,value: *const std::os::raw::c_uint) -> (),
    pub glClearColor: unsafe extern "stdcall" fn(red: f32,green: f32,blue: f32,alpha: f32) -> (),
    pub glClearDepthf: unsafe extern "stdcall" fn(d: f32) -> (),
    pub glClearStencil: unsafe extern "stdcall" fn(s: std::os::raw::c_int) -> (),
    pub glClientWaitSync: unsafe extern "stdcall" fn(sync: *mut (),flags: std::os::raw::c_uint,timeout: u64) -> std::os::raw::c_uint,
    pub glColorMask: unsafe extern "stdcall" fn(red: u8,green: u8,blue: u8,alpha: u8) -> (),
    pub glCompileShader: unsafe extern "stdcall" fn(shader: std::os::raw::c_uint) -> (),
    pub glCompressedTexImage2D: unsafe extern "stdcall" fn(target: std::os::raw::c_uint,level: std::os::raw::c_int,internalformat: std::os::raw::c_uint,width: std::os::raw::c_int,height: std::os::raw::c_int,border: std::os::raw::c_int,imageSize: std::os::raw::c_int,data: *const ()) -> (),
    pub glCompressedTexImage3D: unsafe extern "stdcall" fn(target: std::os::raw::c_uint,level: std::os::raw::c_int,internalformat: std::os::raw::c_uint,width: std::os::raw::c_int,height: std::os::raw::c_int,depth: std::os::raw::c_int,border: std::os::raw::c_int,imageSize: std::os::raw::c_int,data: *const ()) -> (),
    pub glCompressedTexSubImage2D: unsafe extern "stdcall" fn(target: std::os::raw::c_uint,level: std::os::raw::c_int,xoffset: std::os::raw::c_int,yoffset: std::os::raw::c_int,width: std::os::raw::c_int,height: std::os::raw::c_int,format: std::os::raw::c_uint,imageSize: std::os::raw::c_int,data: *const ()) -> (),
    pub glCompressedTexSubImage3D: unsafe extern "stdcall" fn(target: std::os::raw::c_uint,level: std::os::raw::c_int,xoffset: std::os::raw::c_int,yoffset: std::os::raw::c_int,zoffset: std::os::raw::c_int,width: std::os::raw::c_int,height: std::os::raw::c_int,depth: std::os::raw::c_int,format: std::os::raw::c_uint,imageSize: std::os::raw::c_int,data: *const ()) -> (),
    pub glCopyBufferSubData: unsafe extern "stdcall" fn(readTarget: std::os::raw::c_uint,writeTarget: std::os::raw::c_uint,readOffset: isize,writeOffset: isize,size: isize) -> (),
    pub glCopyTexImage2D: unsafe extern "stdcall" fn(target: std::os::raw::c_uint,level: std::os::raw::c_int,internalformat: std::os::raw::c_uint,x: std::os::raw::c_int,y: std::os::raw::c_int,width: std::os::raw::c_int,height: std::os::raw::c_int,border: std::os::raw::c_int) -> (),
    pub glCopyTexSubImage2D: unsafe extern "stdcall" fn(target: std::os::raw::c_uint,level: std::os::raw::c_int,xoffset: std::os::raw::c_int,yoffset: std::os::raw::c_int,x: std::os::raw::c_int,y: std::os::raw::c_int,width: std::os::raw::c_int,height: std::os::raw::c_int) -> (),
    pub glCopyTexSubImage3D: unsafe extern "stdcall" fn(target: std::os::raw::c_uint,level: std::os::raw::c_int,xoffset: std::os::raw::c_int,yoffset: std::os::raw::c_int,zoffset: std::os::raw::c_int,x: std::os::raw::c_int,y: std::os::raw::c_int,width: std::os::raw::c_int,height: std::os::raw::c_int) -> (),
    pub glCreateProgram: unsafe extern "stdcall" fn() -> std::os::raw::c_uint,
    pub glCreateShader: unsafe extern "stdcall" fn(_type: std::os::raw::c_uint) -> std::os::raw::c_uint,
    pub glCullFace: unsafe extern "stdcall" fn(mode: std::os::raw::c_uint) -> (),
    pub glDeleteBuffers: unsafe extern "stdcall" fn(n: std::os::raw::c_int,buffers: *const std::os::raw::c_uint) -> (),
    pub glDeleteFramebuffers: unsafe extern "stdcall" fn(n: std::os::raw::c_int,framebuffers: *const std::os::raw::c_uint) -> (),
    pub glDeleteProgram: unsafe extern "stdcall" fn(program: std::os::raw::c_uint) -> (),
    pub glDeleteQueries: unsafe extern "stdcall" fn(n: std::os::raw::c_int,ids: *const std::os::raw::c_uint) -> (),
    pub glDeleteRenderbuffers: unsafe extern "stdcall" fn(n: std::os::raw::c_int,renderbuffers: *const std::os::raw::c_uint) -> (),
    pub glDeleteSamplers: unsafe extern "stdcall" fn(count: std::os::raw::c_int,samplers: *const std::os::raw::c_uint) -> (),
    pub glDeleteShader: unsafe extern "stdcall" fn(shader: std::os::raw::c_uint) -> (),
    pub glDeleteSync: unsafe extern "stdcall" fn(sync: *mut ()) -> (),
    pub glDeleteTextures: unsafe extern "stdcall" fn(n: std::os::raw::c_int,textures: *const std::os::raw::c_uint) -> (),
    pub glDeleteTransformFeedbacks: unsafe extern "stdcall" fn(n: std::os::raw::c_int,ids: *const std::os::raw::c_uint) -> (),
    pub glDeleteVertexArrays: unsafe extern "stdcall" fn(n: std::os::raw::c_int,arrays: *const std::os::raw::c_uint) -> (),
    pub glDeleteVertexArraysOES: unsafe extern "stdcall" fn(n: std::os::raw::c_int,arrays: *const std::os::raw::c_uint) -> (),
    pub glDepthFunc: unsafe extern "stdcall" fn(func: std::os::raw::c_uint) -> (),
    pub glDepthMask: unsafe extern "stdcall" fn(flag: u8) -> (),
    pub glDepthRangef: unsafe extern "stdcall" fn(n: f32,f: f32) -> (),
    pub glDetachShader: unsafe extern "stdcall" fn(program: std::os::raw::c_uint,shader: std::os::raw::c_uint) -> (),
    pub glDisable: unsafe extern "stdcall" fn(cap: std::os::raw::c_uint) -> (),
    pub glDisableVertexAttribArray: unsafe extern "stdcall" fn(index: std::os::raw::c_uint) -> (),
    pub glDrawArrays: unsafe extern "stdcall" fn(mode: std::os::raw::c_uint,first: std::os::raw::c_int,count: std::os::raw::c_int) -> (),
    pub glDrawArraysInstanced: unsafe extern "stdcall" fn(mode: std::os::raw::c_uint,first: std::os::raw::c_int,count: std::os::raw::c_int,instancecount: std::os::raw::c_int) -> (),
    pub glDrawBuffers: unsafe extern "stdcall" fn(n: std::os::raw::c_int,bufs: *const std::os::raw::c_uint) -> (),
    pub glDrawElements: unsafe extern "stdcall" fn(mode: std::os::raw::c_uint,count: std::os::raw::c_int,_type: std::os::raw::c_uint,indices: *const ()) -> (),
    pub glDrawElementsInstanced: unsafe extern "stdcall" fn(mode: std::os::raw::c_uint,count: std::os::raw::c_int,_type: std::os::raw::c_uint,indices: *const (),instancecount: std::os::raw::c_int) -> (),
    pub glDrawRangeElements: unsafe extern "stdcall" fn(mode: std::os::raw::c_uint,start: std::os::raw::c_uint,end: std::os::raw::c_uint,count: std::os::raw::c_int,_type: std::os::raw::c_uint,indices: *const ()) -> (),
    pub glEnable: unsafe extern "stdcall" fn(cap: std::os::raw::c_uint) -> (),
    pub glEnableVertexAttribArray: unsafe extern "stdcall" fn(index: std::os::raw::c_uint) -> (),
    pub glEndQuery: unsafe extern "stdcall" fn(target: std::os::raw::c_uint) -> (),
    pub glEndTransformFeedback: unsafe extern "stdcall" fn() -> (),
    pub glFenceSync: unsafe extern "stdcall" fn(condition: std::os::raw::c_uint,flags: std::os::raw::c_uint) -> *mut (),
    pub glFinish: unsafe extern "stdcall" fn() -> (),
    pub glFlush: unsafe extern "stdcall" fn() -> (),
    pub glFlushMappedBufferRange: unsafe extern "stdcall" fn(target: std::os::raw::c_uint,offset: isize,length: isize) -> (),
    pub glFramebufferRenderbuffer: unsafe extern "stdcall" fn(target: std::os::raw::c_uint,attachment: std::os::raw::c_uint,renderbuffertarget: std::os::raw::c_uint,renderbuffer: std::os::raw::c_uint) -> (),
    pub glFramebufferTexture2D: unsafe extern "stdcall" fn(target: std::os::raw::c_uint,attachment: std::os::raw::c_uint,textarget: std::os::raw::c_uint,texture: std::os::raw::c_uint,level: std::os::raw::c_int) -> (),
    pub glFramebufferTextureLayer: unsafe extern "stdcall" fn(target: std::os::raw::c_uint,attachment: std::os::raw::c_uint,texture: std::os::raw::c_uint,level: std::os::raw::c_int,layer: std::os::raw::c_int) -> (),
    pub glFrontFace: unsafe extern "stdcall" fn(mode: std::os::raw::c_uint) -> (),
    pub glGenBuffers: unsafe extern "stdcall" fn(n: std::os::raw::c_int,buffers: *mut std::os::raw::c_uint) -> (),
    pub glGenFramebuffers: unsafe extern "stdcall" fn(n: std::os::raw::c_int,framebuffers: *mut std::os::raw::c_uint) -> (),
    pub glGenQueries: unsafe extern "stdcall" fn(n: std::os::raw::c_int,ids: *mut std::os::raw::c_uint) -> (),
    pub glGenRenderbuffers: unsafe extern "stdcall" fn(n: std::os::raw::c_int,renderbuffers: *mut std::os::raw::c_uint) -> (),
    pub glGenSamplers: unsafe extern "stdcall" fn(count: std::os::raw::c_int,samplers: *mut std::os::raw::c_uint) -> (),
    pub glGenTextures: unsafe extern "stdcall" fn(n: std::os::raw::c_int,textures: *mut std::os::raw::c_uint) -> (),
    pub glGenTransformFeedbacks: unsafe extern "stdcall" fn(n: std::os::raw::c_int,ids: *mut std::os::raw::c_uint) -> (),
    pub glGenVertexArrays: unsafe extern "stdcall" fn(n: std::os::raw::c_int,arrays: *mut std::os::raw::c_uint) -> (),
    pub glGenVertexArraysOES: unsafe extern "stdcall" fn(n: std::os::raw::c_int,arrays: *mut std::os::raw::c_uint) -> (),
    pub glGenerateMipmap: unsafe extern "stdcall" fn(target: std::os::raw::c_uint) -> (),
    pub glGetActiveAttrib: unsafe extern "stdcall" fn(program: std::os::raw::c_uint,index: std::os::raw::c_uint,bufSize: std::os::raw::c_int,length: *mut std::os::raw::c_int,size: *mut std::os::raw::c_int,_type: *mut std::os::raw::c_uint,name: *mut i8) -> (),
    pub glGetActiveUniform: unsafe extern "stdcall" fn(program: std::os::raw::c_uint,index: std::os::raw::c_uint,bufSize: std::os::raw::c_int,length: *mut std::os::raw::c_int,size: *mut std::os::raw::c_int,_type: *mut std::os::raw::c_uint,name: *mut i8) -> (),
    pub glGetActiveUniformBlockName: unsafe extern "stdcall" fn(program: std::os::raw::c_uint,uniformBlockIndex: std::os::raw::c_uint,bufSize: std::os::raw::c_int,length: *mut std::os::raw::c_int,uniformBlockName: *mut i8) -> (),
    pub glGetActiveUniformBlockiv: unsafe extern "stdcall" fn(program: std::os::raw::c_uint,uniformBlockIndex: std::os::raw::c_uint,pname: std::os::raw::c_uint,params: *mut std::os::raw::c_int) -> (),
    pub glGetActiveUniformsiv: unsafe extern "stdcall" fn(program: std::os::raw::c_uint,uniformCount: std::os::raw::c_int,uniformIndices: *const std::os::raw::c_uint,pname: std::os::raw::c_uint,params: *mut std::os::raw::c_int) -> (),
    pub glGetAttachedShaders: unsafe extern "stdcall" fn(program: std::os::raw::c_uint,maxCount: std::os::raw::c_int,count: *mut std::os::raw::c_int,shaders: *mut std::os::raw::c_uint) -> (),
    pub glGetAttribLocation: unsafe extern "stdcall" fn(program: std::os::raw::c_uint,name: *const i8) -> std::os::raw::c_int,
    pub glGetBooleanv: unsafe extern "stdcall" fn(pname: std::os::raw::c_uint,data: *mut u8) -> (),
    pub glGetBufferParameteri64v: unsafe extern "stdcall" fn(target: std::os::raw::c_uint,pname: std::os::raw::c_uint,params: *mut i64) -> (),
    pub glGetBufferParameteriv: unsafe extern "stdcall" fn(target: std::os::raw::c_uint,pname: std::os::raw::c_uint,params: *mut std::os::raw::c_int) -> (),
    pub glGetBufferPointerv: unsafe extern "stdcall" fn(target: std::os::raw::c_uint,pname: std::os::raw::c_uint,params: *mut *mut ()) -> (),
    pub glGetError: unsafe extern "stdcall" fn() -> std::os::raw::c_uint,
    pub glGetFloatv: unsafe extern "stdcall" fn(pname: std::os::raw::c_uint,data: *mut f32) -> (),
    pub glGetFragDataLocation: unsafe extern "stdcall" fn(program: std::os::raw::c_uint,name: *const i8) -> std::os::raw::c_int,
    pub glGetFramebufferAttachmentParameteriv: unsafe extern "stdcall" fn(target: std::os::raw::c_uint,attachment: std::os::raw::c_uint,pname: std::os::raw::c_uint,params: *mut std::os::raw::c_int) -> (),
    pub glGetInteger64i_v: unsafe extern "stdcall" fn(target: std::os::raw::c_uint,index: std::os::raw::c_uint,data: *mut i64) -> (),
    pub glGetInteger64v: unsafe extern "stdcall" fn(pname: std::os::raw::c_uint,data: *mut i64) -> (),
    pub glGetIntegeri_v: unsafe extern "stdcall" fn(target: std::os::raw::c_uint,index: std::os::raw::c_uint,data: *mut std::os::raw::c_int) -> (),
    pub glGetIntegerv: unsafe extern "stdcall" fn(pname: std::os::raw::c_uint,data: *mut std::os::raw::c_int) -> (),
    pub glGetInternalformativ: unsafe extern "stdcall" fn(target: std::os::raw::c_uint,internalformat: std::os::raw::c_uint,pname: std::os::raw::c_uint,count: std::os::raw::c_int,params: *mut std::os::raw::c_int) -> (),
    pub glGetProgramBinary: unsafe extern "stdcall" fn(program: std::os::raw::c_uint,bufSize: std::os::raw::c_int,length: *mut std::os::raw::c_int,binaryFormat: *mut std::os::raw::c_uint,binary: *mut ()) -> (),
    pub glGetProgramInfoLog: unsafe extern "stdcall" fn(program: std::os::raw::c_uint,bufSize: std::os::raw::c_int,length: *mut std::os::raw::c_int,infoLog: *mut i8) -> (),
    pub glGetProgramiv: unsafe extern "stdcall" fn(program: std::os::raw::c_uint,pname: std::os::raw::c_uint,params: *mut std::os::raw::c_int) -> (),
    pub glGetQueryObjectuiv: unsafe extern "stdcall" fn(id: std::os::raw::c_uint,pname: std::os::raw::c_uint,params: *mut std::os::raw::c_uint) -> (),
    pub glGetQueryiv: unsafe extern "stdcall" fn(target: std::os::raw::c_uint,pname: std::os::raw::c_uint,params: *mut std::os::raw::c_int) -> (),
    pub glGetRenderbufferParameteriv: unsafe extern "stdcall" fn(target: std::os::raw::c_uint,pname: std::os::raw::c_uint,params: *mut std::os::raw::c_int) -> (),
    pub glGetSamplerParameterfv: unsafe extern "stdcall" fn(sampler: std::os::raw::c_uint,pname: std::os::raw::c_uint,params: *mut f32) -> (),
    pub glGetSamplerParameteriv: unsafe extern "stdcall" fn(sampler: std::os::raw::c_uint,pname: std::os::raw::c_uint,params: *mut std::os::raw::c_int) -> (),
    pub glGetShaderInfoLog: unsafe extern "stdcall" fn(shader: std::os::raw::c_uint,bufSize: std::os::raw::c_int,length: *mut std::os::raw::c_int,infoLog: *mut i8) -> (),
    pub glGetShaderPrecisionFormat: unsafe extern "stdcall" fn(shadertype: std::os::raw::c_uint,precisiontype: std::os::raw::c_uint,range: *mut std::os::raw::c_int,precision: *mut std::os::raw::c_int) -> (),
    pub glGetShaderSource: unsafe extern "stdcall" fn(shader: std::os::raw::c_uint,bufSize: std::os::raw::c_int,length: *mut std::os::raw::c_int,source: *mut i8) -> (),
    pub glGetShaderiv: unsafe extern "stdcall" fn(shader: std::os::raw::c_uint,pname: std::os::raw::c_uint,params: *mut std::os::raw::c_int) -> (),
    pub glGetString: unsafe extern "stdcall" fn(name: std::os::raw::c_uint) -> *const u8,
    pub glGetStringi: unsafe extern "stdcall" fn(name: std::os::raw::c_uint,index: std::os::raw::c_uint) -> *const u8,
    pub glGetSynciv: unsafe extern "stdcall" fn(sync: *mut (),pname: std::os::raw::c_uint,count: std::os::raw::c_int,length: *mut std::os::raw::c_int,values: *mut std::os::raw::c_int) -> (),
    pub glGetTexParameterfv: unsafe extern "stdcall" fn(target: std::os::raw::c_uint,pname: std::os::raw::c_uint,params: *mut f32) -> (),
    pub glGetTexParameteriv: unsafe extern "stdcall" fn(target: std::os::raw::c_uint,pname: std::os::raw::c_uint,params: *mut std::os::raw::c_int) -> (),
    pub glGetTransformFeedbackVarying: unsafe extern "stdcall" fn(program: std::os::raw::c_uint,index: std::os::raw::c_uint,bufSize: std::os::raw::c_int,length: *mut std::os::raw::c_int,size: *mut std::os::raw::c_int,_type: *mut std::os::raw::c_uint,name: *mut i8) -> (),
    pub glGetUniformBlockIndex: unsafe extern "stdcall" fn(program: std::os::raw::c_uint,uniformBlockName: *const i8) -> std::os::raw::c_uint,
    pub glGetUniformIndices: unsafe extern "stdcall" fn(program: std::os::raw::c_uint,uniformCount: std::os::raw::c_int,uniformNames: *const *const i8,uniformIndices: *mut std::os::raw::c_uint) -> (),
    pub glGetUniformLocation: unsafe extern "stdcall" fn(program: std::os::raw::c_uint,name: *const i8) -> std::os::raw::c_int,
    pub glGetUniformfv: unsafe extern "stdcall" fn(program: std::os::raw::c_uint,location: std::os::raw::c_int,params: *mut f32) -> (),
    pub glGetUniformiv: unsafe extern "stdcall" fn(program: std::os::raw::c_uint,location: std::os::raw::c_int,params: *mut std::os::raw::c_int) -> (),
    pub glGetUniformuiv: unsafe extern "stdcall" fn(program: std::os::raw::c_uint,location: std::os::raw::c_int,params: *mut std::os::raw::c_uint) -> (),
    pub glGetVertexAttribIiv: unsafe extern "stdcall" fn(index: std::os::raw::c_uint,pname: std::os::raw::c_uint,params: *mut std::os::raw::c_int) -> (),
    pub glGetVertexAttribIuiv: unsafe extern "stdcall" fn(index: std::os::raw::c_uint,pname: std::os::raw::c_uint,params: *mut std::os::raw::c_uint) -> (),
    pub glGetVertexAttribPointerv: unsafe extern "stdcall" fn(index: std::os::raw::c_uint,pname: std::os::raw::c_uint,pointer: *mut *mut ()) -> (),
    pub glGetVertexAttribfv: unsafe extern "stdcall" fn(index: std::os::raw::c_uint,pname: std::os::raw::c_uint,params: *mut f32) -> (),
    pub glGetVertexAttribiv: unsafe extern "stdcall" fn(index: std::os::raw::c_uint,pname: std::os::raw::c_uint,params: *mut std::os::raw::c_int) -> (),
    pub glHint: unsafe extern "stdcall" fn(target: std::os::raw::c_uint,mode: std::os::raw::c_uint) -> (),
    pub glInvalidateFramebuffer: unsafe extern "stdcall" fn(target: std::os::raw::c_uint,numAttachments: std::os::raw::c_int,attachments: *const std::os::raw::c_uint) -> (),
    pub glInvalidateSubFramebuffer: unsafe extern "stdcall" fn(target: std::os::raw::c_uint,numAttachments: std::os::raw::c_int,attachments: *const std::os::raw::c_uint,x: std::os::raw::c_int,y: std::os::raw::c_int,width: std::os::raw::c_int,height: std::os::raw::c_int) -> (),
    pub glIsBuffer: unsafe extern "stdcall" fn(buffer: std::os::raw::c_uint) -> u8,
    pub glIsEnabled: unsafe extern "stdcall" fn(cap: std::os::raw::c_uint) -> u8,
    pub glIsFramebuffer: unsafe extern "stdcall" fn(framebuffer: std::os::raw::c_uint) -> u8,
    pub glIsProgram: unsafe extern "stdcall" fn(program: std::os::raw::c_uint) -> u8,
    pub glIsQuery: unsafe extern "stdcall" fn(id: std::os::raw::c_uint) -> u8,
    pub glIsRenderbuffer: unsafe extern "stdcall" fn(renderbuffer: std::os::raw::c_uint) -> u8,
    pub glIsSampler: unsafe extern "stdcall" fn(sampler: std::os::raw::c_uint) -> u8,
    pub glIsShader: unsafe extern "stdcall" fn(shader: std::os::raw::c_uint) -> u8,
    pub glIsSync: unsafe extern "stdcall" fn(sync: *mut ()) -> u8,
    pub glIsTexture: unsafe extern "stdcall" fn(texture: std::os::raw::c_uint) -> u8,
    pub glIsTransformFeedback: unsafe extern "stdcall" fn(id: std::os::raw::c_uint) -> u8,
    pub glIsVertexArray: unsafe extern "stdcall" fn(array: std::os::raw::c_uint) -> u8,
    pub glIsVertexArrayOES: unsafe extern "stdcall" fn(array: std::os::raw::c_uint) -> u8,
    pub glLineWidth: unsafe extern "stdcall" fn(width: f32) -> (),
    pub glLinkProgram: unsafe extern "stdcall" fn(program: std::os::raw::c_uint) -> (),
    pub glMapBufferRange: unsafe extern "stdcall" fn(target: std::os::raw::c_uint,offset: isize,length: isize,access: std::os::raw::c_uint) -> *mut (),
    pub glPauseTransformFeedback: unsafe extern "stdcall" fn() -> (),
    pub glPixelStorei: unsafe extern "stdcall" fn(pname: std::os::raw::c_uint,param: std::os::raw::c_int) -> (),
    pub glPolygonOffset: unsafe extern "stdcall" fn(factor: f32,units: f32) -> (),
    pub glProgramBinary: unsafe extern "stdcall" fn(program: std::os::raw::c_uint,binaryFormat: std::os::raw::c_uint,binary: *const (),length: std::os::raw::c_int) -> (),
    pub glProgramParameteri: unsafe extern "stdcall" fn(program: std::os::raw::c_uint,pname: std::os::raw::c_uint,value: std::os::raw::c_int) -> (),
    pub glReadBuffer: unsafe extern "stdcall" fn(src: std::os::raw::c_uint) -> (),
    pub glReadPixels: unsafe extern "stdcall" fn(x: std::os::raw::c_int,y: std::os::raw::c_int,width: std::os::raw::c_int,height: std::os::raw::c_int,format: std::os::raw::c_uint,_type: std::os::raw::c_uint,pixels: *mut ()) -> (),
    pub glReleaseShaderCompiler: unsafe extern "stdcall" fn() -> (),
    pub glRenderbufferStorage: unsafe extern "stdcall" fn(target: std::os::raw::c_uint,internalformat: std::os::raw::c_uint,width: std::os::raw::c_int,height: std::os::raw::c_int) -> (),
    pub glRenderbufferStorageMultisample: unsafe extern "stdcall" fn(target: std::os::raw::c_uint,samples: std::os::raw::c_int,internalformat: std::os::raw::c_uint,width: std::os::raw::c_int,height: std::os::raw::c_int) -> (),
    pub glResumeTransformFeedback: unsafe extern "stdcall" fn() -> (),
    pub glSampleCoverage: unsafe extern "stdcall" fn(value: f32,invert: u8) -> (),
    pub glSamplerParameterf: unsafe extern "stdcall" fn(sampler: std::os::raw::c_uint,pname: std::os::raw::c_uint,param: f32) -> (),
    pub glSamplerParameterfv: unsafe extern "stdcall" fn(sampler: std::os::raw::c_uint,pname: std::os::raw::c_uint,param: *const f32) -> (),
    pub glSamplerParameteri: unsafe extern "stdcall" fn(sampler: std::os::raw::c_uint,pname: std::os::raw::c_uint,param: std::os::raw::c_int) -> (),
    pub glSamplerParameteriv: unsafe extern "stdcall" fn(sampler: std::os::raw::c_uint,pname: std::os::raw::c_uint,param: *const std::os::raw::c_int) -> (),
    pub glScissor: unsafe extern "stdcall" fn(x: std::os::raw::c_int,y: std::os::raw::c_int,width: std::os::raw::c_int,height: std::os::raw::c_int) -> (),
    pub glShaderBinary: unsafe extern "stdcall" fn(count: std::os::raw::c_int,shaders: *const std::os::raw::c_uint,binaryFormat: std::os::raw::c_uint,binary: *const (),length: std::os::raw::c_int) -> (),
    pub glShaderSource: unsafe extern "stdcall" fn(shader: std::os::raw::c_uint,count: std::os::raw::c_int,string: *const *const i8,length: *const std::os::raw::c_int) -> (),
    pub glStencilFunc: unsafe extern "stdcall" fn(func: std::os::raw::c_uint,_ref: std::os::raw::c_int,mask: std::os::raw::c_uint) -> (),
    pub glStencilFuncSeparate: unsafe extern "stdcall" fn(face: std::os::raw::c_uint,func: std::os::raw::c_uint,_ref: std::os::raw::c_int,mask: std::os::raw::c_uint) -> (),
    pub glStencilMask: unsafe extern "stdcall" fn(mask: std::os::raw::c_uint) -> (),
    pub glStencilMaskSeparate: unsafe extern "stdcall" fn(face: std::os::raw::c_uint,mask: std::os::raw::c_uint) -> (),
    pub glStencilOp: unsafe extern "stdcall" fn(fail: std::os::raw::c_uint,zfail: std::os::raw::c_uint,zpass: std::os::raw::c_uint) -> (),
    pub glStencilOpSeparate: unsafe extern "stdcall" fn(face: std::os::raw::c_uint,sfail: std::os::raw::c_uint,dpfail: std::os::raw::c_uint,dppass: std::os::raw::c_uint) -> (),
    pub glTexImage2D: unsafe extern "stdcall" fn(target: std::os::raw::c_uint,level: std::os::raw::c_int,internalformat: std::os::raw::c_int,width: std::os::raw::c_int,height: std::os::raw::c_int,border: std::os::raw::c_int,format: std::os::raw::c_uint,_type: std::os::raw::c_uint,pixels: *const ()) -> (),
    pub glTexImage3D: unsafe extern "stdcall" fn(target: std::os::raw::c_uint,level: std::os::raw::c_int,internalformat: std::os::raw::c_int,width: std::os::raw::c_int,height: std::os::raw::c_int,depth: std::os::raw::c_int,border: std::os::raw::c_int,format: std::os::raw::c_uint,_type: std::os::raw::c_uint,pixels: *const ()) -> (),
    pub glTexParameterf: unsafe extern "stdcall" fn(target: std::os::raw::c_uint,pname: std::os::raw::c_uint,param: f32) -> (),
    pub glTexParameterfv: unsafe extern "stdcall" fn(target: std::os::raw::c_uint,pname: std::os::raw::c_uint,params: *const f32) -> (),
    pub glTexParameteri: unsafe extern "stdcall" fn(target: std::os::raw::c_uint,pname: std::os::raw::c_uint,param: std::os::raw::c_int) -> (),
    pub glTexParameteriv: unsafe extern "stdcall" fn(target: std::os::raw::c_uint,pname: std::os::raw::c_uint,params: *const std::os::raw::c_int) -> (),
    pub glTexStorage2D: unsafe extern "stdcall" fn(target: std::os::raw::c_uint,levels: std::os::raw::c_int,internalformat: std::os::raw::c_uint,width: std::os::raw::c_int,height: std::os::raw::c_int) -> (),
    pub glTexStorage3D: unsafe extern "stdcall" fn(target: std::os::raw::c_uint,levels: std::os::raw::c_int,internalformat: std::os::raw::c_uint,width: std::os::raw::c_int,height: std::os::raw::c_int,depth: std::os::raw::c_int) -> (),
    pub glTexSubImage2D: unsafe extern "stdcall" fn(target: std::os::raw::c_uint,level: std::os::raw::c_int,xoffset: std::os::raw::c_int,yoffset: std::os::raw::c_int,width: std::os::raw::c_int,height: std::os::raw::c_int,format: std::os::raw::c_uint,_type: std::os::raw::c_uint,pixels: *const ()) -> (),
    pub glTexSubImage3D: unsafe extern "stdcall" fn(target: std::os::raw::c_uint,level: std::os::raw::c_int,xoffset: std::os::raw::c_int,yoffset: std::os::raw::c_int,zoffset: std::os::raw::c_int,width: std::os::raw::c_int,height: std::os::raw::c_int,depth: std::os::raw::c_int,format: std::os::raw::c_uint,_type: std::os::raw::c_uint,pixels: *const ()) -> (),
    pub glTransformFeedbackVaryings: unsafe extern "stdcall" fn(program: std::os::raw::c_uint,count: std::os::raw::c_int,varyings: *const *const i8,bufferMode: std::os::raw::c_uint) -> (),
    pub glUniform1f: unsafe extern "stdcall" fn(location: std::os::raw::c_int,v0: f32) -> (),
    pub glUniform1fv: unsafe extern "stdcall" fn(location: std::os::raw::c_int,count: std::os::raw::c_int,value: *const f32) -> (),
    pub glUniform1i: unsafe extern "stdcall" fn(location: std::os::raw::c_int,v0: std::os::raw::c_int) -> (),
    pub glUniform1iv: unsafe extern "stdcall" fn(location: std::os::raw::c_int,count: std::os::raw::c_int,value: *const std::os::raw::c_int) -> (),
    pub glUniform1ui: unsafe extern "stdcall" fn(location: std::os::raw::c_int,v0: std::os::raw::c_uint) -> (),
    pub glUniform1uiv: unsafe extern "stdcall" fn(location: std::os::raw::c_int,count: std::os::raw::c_int,value: *const std::os::raw::c_uint) -> (),
    pub glUniform2f: unsafe extern "stdcall" fn(location: std::os::raw::c_int,v0: f32,v1: f32) -> (),
    pub glUniform2fv: unsafe extern "stdcall" fn(location: std::os::raw::c_int,count: std::os::raw::c_int,value: *const f32) -> (),
    pub glUniform2i: unsafe extern "stdcall" fn(location: std::os::raw::c_int,v0: std::os::raw::c_int,v1: std::os::raw::c_int) -> (),
    pub glUniform2iv: unsafe extern "stdcall" fn(location: std::os::raw::c_int,count: std::os::raw::c_int,value: *const std::os::raw::c_int) -> (),
    pub glUniform2ui: unsafe extern "stdcall" fn(location: std::os::raw::c_int,v0: std::os::raw::c_uint,v1: std::os::raw::c_uint) -> (),
    pub glUniform2uiv: unsafe extern "stdcall" fn(location: std::os::raw::c_int,count: std::os::raw::c_int,value: *const std::os::raw::c_uint) -> (),
    pub glUniform3f: unsafe extern "stdcall" fn(location: std::os::raw::c_int,v0: f32,v1: f32,v2: f32) -> (),
    pub glUniform3fv: unsafe extern "stdcall" fn(location: std::os::raw::c_int,count: std::os::raw::c_int,value: *const f32) -> (),
    pub glUniform3i: unsafe extern "stdcall" fn(location: std::os::raw::c_int,v0: std::os::raw::c_int,v1: std::os::raw::c_int,v2: std::os::raw::c_int) -> (),
    pub glUniform3iv: unsafe extern "stdcall" fn(location: std::os::raw::c_int,count: std::os::raw::c_int,value: *const std::os::raw::c_int) -> (),
    pub glUniform3ui: unsafe extern "stdcall" fn(location: std::os::raw::c_int,v0: std::os::raw::c_uint,v1: std::os::raw::c_uint,v2: std::os::raw::c_uint) -> (),
    pub glUniform3uiv: unsafe extern "stdcall" fn(location: std::os::raw::c_int,count: std::os::raw::c_int,value: *const std::os::raw::c_uint) -> (),
    pub glUniform4f: unsafe extern "stdcall" fn(location: std::os::raw::c_int,v0: f32,v1: f32,v2: f32,v3: f32) -> (),
    pub glUniform4fv: unsafe extern "stdcall" fn(location: std::os::raw::c_int,count: std::os::raw::c_int,value: *const f32) -> (),
    pub glUniform4i: unsafe extern "stdcall" fn(location: std::os::raw::c_int,v0: std::os::raw::c_int,v1: std::os::raw::c_int,v2: std::os::raw::c_int,v3: std::os::raw::c_int) -> (),
    pub glUniform4iv: unsafe extern "stdcall" fn(location: std::os::raw::c_int,count: std::os::raw::c_int,value: *const std::os::raw::c_int) -> (),
    pub glUniform4ui: unsafe extern "stdcall" fn(location: std::os::raw::c_int,v0: std::os::raw::c_uint,v1: std::os::raw::c_uint,v2: std::os::raw::c_uint,v3: std::os::raw::c_uint) -> (),
    pub glUniform4uiv: unsafe extern "stdcall" fn(location: std::os::raw::c_int,count: std::os::raw::c_int,value: *const std::os::raw::c_uint) -> (),
    pub glUniformBlockBinding: unsafe extern "stdcall" fn(program: std::os::raw::c_uint,uniformBlockIndex: std::os::raw::c_uint,uniformBlockBinding: std::os::raw::c_uint) -> (),
    pub glUniformMatrix2fv: unsafe extern "stdcall" fn(location: std::os::raw::c_int,count: std::os::raw::c_int,transpose: u8,value: *const f32) -> (),
    pub glUniformMatrix2x3fv: unsafe extern "stdcall" fn(location: std::os::raw::c_int,count: std::os::raw::c_int,transpose: u8,value: *const f32) -> (),
    pub glUniformMatrix2x4fv: unsafe extern "stdcall" fn(location: std::os::raw::c_int,count: std::os::raw::c_int,transpose: u8,value: *const f32) -> (),
    pub glUniformMatrix3fv: unsafe extern "stdcall" fn(location: std::os::raw::c_int,count: std::os::raw::c_int,transpose: u8,value: *const f32) -> (),
    pub glUniformMatrix3x2fv: unsafe extern "stdcall" fn(location: std::os::raw::c_int,count: std::os::raw::c_int,transpose: u8,value: *const f32) -> (),
    pub glUniformMatrix3x4fv: unsafe extern "stdcall" fn(location: std::os::raw::c_int,count: std::os::raw::c_int,transpose: u8,value: *const f32) -> (),
    pub glUniformMatrix4fv: unsafe extern "stdcall" fn(location: std::os::raw::c_int,count: std::os::raw::c_int,transpose: u8,value: *const f32) -> (),
    pub glUniformMatrix4x2fv: unsafe extern "stdcall" fn(location: std::os::raw::c_int,count: std::os::raw::c_int,transpose: u8,value: *const f32) -> (),
    pub glUniformMatrix4x3fv: unsafe extern "stdcall" fn(location: std::os::raw::c_int,count: std::os::raw::c_int,transpose: u8,value: *const f32) -> (),
    pub glUnmapBuffer: unsafe extern "stdcall" fn(target: std::os::raw::c_uint) -> u8,
    pub glUseProgram: unsafe extern "stdcall" fn(program: std::os::raw::c_uint) -> (),
    pub glValidateProgram: unsafe extern "stdcall" fn(program: std::os::raw::c_uint) -> (),
    pub glVertexAttrib1f: unsafe extern "stdcall" fn(index: std::os::raw::c_uint,x: f32) -> (),
    pub glVertexAttrib1fv: unsafe extern "stdcall" fn(index: std::os::raw::c_uint,v: *const f32) -> (),
    pub glVertexAttrib2f: unsafe extern "stdcall" fn(index: std::os::raw::c_uint,x: f32,y: f32) -> (),
    pub glVertexAttrib2fv: unsafe extern "stdcall" fn(index: std::os::raw::c_uint,v: *const f32) -> (),
    pub glVertexAttrib3f: unsafe extern "stdcall" fn(index: std::os::raw::c_uint,x: f32,y: f32,z: f32) -> (),
    pub glVertexAttrib3fv: unsafe extern "stdcall" fn(index: std::os::raw::c_uint,v: *const f32) -> (),
    pub glVertexAttrib4f: unsafe extern "stdcall" fn(index: std::os::raw::c_uint,x: f32,y: f32,z: f32,w: f32) -> (),
    pub glVertexAttrib4fv: unsafe extern "stdcall" fn(index: std::os::raw::c_uint,v: *const f32) -> (),
    pub glVertexAttribDivisor: unsafe extern "stdcall" fn(index: std::os::raw::c_uint,divisor: std::os::raw::c_uint) -> (),
    pub glVertexAttribI4i: unsafe extern "stdcall" fn(index: std::os::raw::c_uint,x: std::os::raw::c_int,y: std::os::raw::c_int,z: std::os::raw::c_int,w: std::os::raw::c_int) -> (),
    pub glVertexAttribI4iv: unsafe extern "stdcall" fn(index: std::os::raw::c_uint,v: *const std::os::raw::c_int) -> (),
    pub glVertexAttribI4ui: unsafe extern "stdcall" fn(index: std::os::raw::c_uint,x: std::os::raw::c_uint,y: std::os::raw::c_uint,z: std::os::raw::c_uint,w: std::os::raw::c_uint) -> (),
    pub glVertexAttribI4uiv: unsafe extern "stdcall" fn(index: std::os::raw::c_uint,v: *const std::os::raw::c_uint) -> (),
    pub glVertexAttribIPointer: unsafe extern "stdcall" fn(index: std::os::raw::c_uint,size: std::os::raw::c_int,_type: std::os::raw::c_uint,stride: std::os::raw::c_int,pointer: *const ()) -> (),
    pub glVertexAttribPointer: unsafe extern "stdcall" fn(index: std::os::raw::c_uint,size: std::os::raw::c_int,_type: std::os::raw::c_uint,normalized: u8,stride: std::os::raw::c_int,pointer: *const ()) -> (),
    pub glViewport: unsafe extern "stdcall" fn(x: std::os::raw::c_int,y: std::os::raw::c_int,width: std::os::raw::c_int,height: std::os::raw::c_int) -> (),
    pub glWaitSync: unsafe extern "stdcall" fn(sync: *mut (),flags: std::os::raw::c_uint,timeout: u64) -> (),
}

#[rustfmt::skip]
impl ProcAddresses {
    #[allow(invalid_value)]
    pub fn new() -> Self {
        unsafe { std::mem::MaybeUninit::<ProcAddresses>::zeroed().assume_init() }
    }

    pub fn fill(&mut self, _context: &mut webrogue_gfx::Context) {
        unsafe {
            self.glActiveTexture = std::mem::transmute(crate::utils::get_proc_address(_context, "glActiveTexture"));
            self.glAttachShader = std::mem::transmute(crate::utils::get_proc_address(_context, "glAttachShader"));
            self.glBeginQuery = std::mem::transmute(crate::utils::get_proc_address(_context, "glBeginQuery"));
            self.glBeginTransformFeedback = std::mem::transmute(crate::utils::get_proc_address(_context, "glBeginTransformFeedback"));
            self.glBindAttribLocation = std::mem::transmute(crate::utils::get_proc_address(_context, "glBindAttribLocation"));
            self.glBindBuffer = std::mem::transmute(crate::utils::get_proc_address(_context, "glBindBuffer"));
            self.glBindBufferBase = std::mem::transmute(crate::utils::get_proc_address(_context, "glBindBufferBase"));
            self.glBindBufferRange = std::mem::transmute(crate::utils::get_proc_address(_context, "glBindBufferRange"));
            self.glBindFramebuffer = std::mem::transmute(crate::utils::get_proc_address(_context, "glBindFramebuffer"));
            self.glBindRenderbuffer = std::mem::transmute(crate::utils::get_proc_address(_context, "glBindRenderbuffer"));
            self.glBindSampler = std::mem::transmute(crate::utils::get_proc_address(_context, "glBindSampler"));
            self.glBindTexture = std::mem::transmute(crate::utils::get_proc_address(_context, "glBindTexture"));
            self.glBindTransformFeedback = std::mem::transmute(crate::utils::get_proc_address(_context, "glBindTransformFeedback"));
            self.glBindVertexArray = std::mem::transmute(crate::utils::get_proc_address(_context, "glBindVertexArray"));
            self.glBindVertexArrayOES = std::mem::transmute(crate::utils::get_proc_address(_context, "glBindVertexArrayOES"));
            self.glBlendColor = std::mem::transmute(crate::utils::get_proc_address(_context, "glBlendColor"));
            self.glBlendEquation = std::mem::transmute(crate::utils::get_proc_address(_context, "glBlendEquation"));
            self.glBlendEquationSeparate = std::mem::transmute(crate::utils::get_proc_address(_context, "glBlendEquationSeparate"));
            self.glBlendFunc = std::mem::transmute(crate::utils::get_proc_address(_context, "glBlendFunc"));
            self.glBlendFuncSeparate = std::mem::transmute(crate::utils::get_proc_address(_context, "glBlendFuncSeparate"));
            self.glBlitFramebuffer = std::mem::transmute(crate::utils::get_proc_address(_context, "glBlitFramebuffer"));
            self.glBufferData = std::mem::transmute(crate::utils::get_proc_address(_context, "glBufferData"));
            self.glBufferSubData = std::mem::transmute(crate::utils::get_proc_address(_context, "glBufferSubData"));
            self.glCheckFramebufferStatus = std::mem::transmute(crate::utils::get_proc_address(_context, "glCheckFramebufferStatus"));
            self.glClear = std::mem::transmute(crate::utils::get_proc_address(_context, "glClear"));
            self.glClearBufferfi = std::mem::transmute(crate::utils::get_proc_address(_context, "glClearBufferfi"));
            self.glClearBufferfv = std::mem::transmute(crate::utils::get_proc_address(_context, "glClearBufferfv"));
            self.glClearBufferiv = std::mem::transmute(crate::utils::get_proc_address(_context, "glClearBufferiv"));
            self.glClearBufferuiv = std::mem::transmute(crate::utils::get_proc_address(_context, "glClearBufferuiv"));
            self.glClearColor = std::mem::transmute(crate::utils::get_proc_address(_context, "glClearColor"));
            self.glClearDepthf = std::mem::transmute(crate::utils::get_proc_address(_context, "glClearDepthf"));
            self.glClearStencil = std::mem::transmute(crate::utils::get_proc_address(_context, "glClearStencil"));
            self.glClientWaitSync = std::mem::transmute(crate::utils::get_proc_address(_context, "glClientWaitSync"));
            self.glColorMask = std::mem::transmute(crate::utils::get_proc_address(_context, "glColorMask"));
            self.glCompileShader = std::mem::transmute(crate::utils::get_proc_address(_context, "glCompileShader"));
            self.glCompressedTexImage2D = std::mem::transmute(crate::utils::get_proc_address(_context, "glCompressedTexImage2D"));
            self.glCompressedTexImage3D = std::mem::transmute(crate::utils::get_proc_address(_context, "glCompressedTexImage3D"));
            self.glCompressedTexSubImage2D = std::mem::transmute(crate::utils::get_proc_address(_context, "glCompressedTexSubImage2D"));
            self.glCompressedTexSubImage3D = std::mem::transmute(crate::utils::get_proc_address(_context, "glCompressedTexSubImage3D"));
            self.glCopyBufferSubData = std::mem::transmute(crate::utils::get_proc_address(_context, "glCopyBufferSubData"));
            self.glCopyTexImage2D = std::mem::transmute(crate::utils::get_proc_address(_context, "glCopyTexImage2D"));
            self.glCopyTexSubImage2D = std::mem::transmute(crate::utils::get_proc_address(_context, "glCopyTexSubImage2D"));
            self.glCopyTexSubImage3D = std::mem::transmute(crate::utils::get_proc_address(_context, "glCopyTexSubImage3D"));
            self.glCreateProgram = std::mem::transmute(crate::utils::get_proc_address(_context, "glCreateProgram"));
            self.glCreateShader = std::mem::transmute(crate::utils::get_proc_address(_context, "glCreateShader"));
            self.glCullFace = std::mem::transmute(crate::utils::get_proc_address(_context, "glCullFace"));
            self.glDeleteBuffers = std::mem::transmute(crate::utils::get_proc_address(_context, "glDeleteBuffers"));
            self.glDeleteFramebuffers = std::mem::transmute(crate::utils::get_proc_address(_context, "glDeleteFramebuffers"));
            self.glDeleteProgram = std::mem::transmute(crate::utils::get_proc_address(_context, "glDeleteProgram"));
            self.glDeleteQueries = std::mem::transmute(crate::utils::get_proc_address(_context, "glDeleteQueries"));
            self.glDeleteRenderbuffers = std::mem::transmute(crate::utils::get_proc_address(_context, "glDeleteRenderbuffers"));
            self.glDeleteSamplers = std::mem::transmute(crate::utils::get_proc_address(_context, "glDeleteSamplers"));
            self.glDeleteShader = std::mem::transmute(crate::utils::get_proc_address(_context, "glDeleteShader"));
            self.glDeleteSync = std::mem::transmute(crate::utils::get_proc_address(_context, "glDeleteSync"));
            self.glDeleteTextures = std::mem::transmute(crate::utils::get_proc_address(_context, "glDeleteTextures"));
            self.glDeleteTransformFeedbacks = std::mem::transmute(crate::utils::get_proc_address(_context, "glDeleteTransformFeedbacks"));
            self.glDeleteVertexArrays = std::mem::transmute(crate::utils::get_proc_address(_context, "glDeleteVertexArrays"));
            self.glDeleteVertexArraysOES = std::mem::transmute(crate::utils::get_proc_address(_context, "glDeleteVertexArraysOES"));
            self.glDepthFunc = std::mem::transmute(crate::utils::get_proc_address(_context, "glDepthFunc"));
            self.glDepthMask = std::mem::transmute(crate::utils::get_proc_address(_context, "glDepthMask"));
            self.glDepthRangef = std::mem::transmute(crate::utils::get_proc_address(_context, "glDepthRangef"));
            self.glDetachShader = std::mem::transmute(crate::utils::get_proc_address(_context, "glDetachShader"));
            self.glDisable = std::mem::transmute(crate::utils::get_proc_address(_context, "glDisable"));
            self.glDisableVertexAttribArray = std::mem::transmute(crate::utils::get_proc_address(_context, "glDisableVertexAttribArray"));
            self.glDrawArrays = std::mem::transmute(crate::utils::get_proc_address(_context, "glDrawArrays"));
            self.glDrawArraysInstanced = std::mem::transmute(crate::utils::get_proc_address(_context, "glDrawArraysInstanced"));
            self.glDrawBuffers = std::mem::transmute(crate::utils::get_proc_address(_context, "glDrawBuffers"));
            self.glDrawElements = std::mem::transmute(crate::utils::get_proc_address(_context, "glDrawElements"));
            self.glDrawElementsInstanced = std::mem::transmute(crate::utils::get_proc_address(_context, "glDrawElementsInstanced"));
            self.glDrawRangeElements = std::mem::transmute(crate::utils::get_proc_address(_context, "glDrawRangeElements"));
            self.glEnable = std::mem::transmute(crate::utils::get_proc_address(_context, "glEnable"));
            self.glEnableVertexAttribArray = std::mem::transmute(crate::utils::get_proc_address(_context, "glEnableVertexAttribArray"));
            self.glEndQuery = std::mem::transmute(crate::utils::get_proc_address(_context, "glEndQuery"));
            self.glEndTransformFeedback = std::mem::transmute(crate::utils::get_proc_address(_context, "glEndTransformFeedback"));
            self.glFenceSync = std::mem::transmute(crate::utils::get_proc_address(_context, "glFenceSync"));
            self.glFinish = std::mem::transmute(crate::utils::get_proc_address(_context, "glFinish"));
            self.glFlush = std::mem::transmute(crate::utils::get_proc_address(_context, "glFlush"));
            self.glFlushMappedBufferRange = std::mem::transmute(crate::utils::get_proc_address(_context, "glFlushMappedBufferRange"));
            self.glFramebufferRenderbuffer = std::mem::transmute(crate::utils::get_proc_address(_context, "glFramebufferRenderbuffer"));
            self.glFramebufferTexture2D = std::mem::transmute(crate::utils::get_proc_address(_context, "glFramebufferTexture2D"));
            self.glFramebufferTextureLayer = std::mem::transmute(crate::utils::get_proc_address(_context, "glFramebufferTextureLayer"));
            self.glFrontFace = std::mem::transmute(crate::utils::get_proc_address(_context, "glFrontFace"));
            self.glGenBuffers = std::mem::transmute(crate::utils::get_proc_address(_context, "glGenBuffers"));
            self.glGenFramebuffers = std::mem::transmute(crate::utils::get_proc_address(_context, "glGenFramebuffers"));
            self.glGenQueries = std::mem::transmute(crate::utils::get_proc_address(_context, "glGenQueries"));
            self.glGenRenderbuffers = std::mem::transmute(crate::utils::get_proc_address(_context, "glGenRenderbuffers"));
            self.glGenSamplers = std::mem::transmute(crate::utils::get_proc_address(_context, "glGenSamplers"));
            self.glGenTextures = std::mem::transmute(crate::utils::get_proc_address(_context, "glGenTextures"));
            self.glGenTransformFeedbacks = std::mem::transmute(crate::utils::get_proc_address(_context, "glGenTransformFeedbacks"));
            self.glGenVertexArrays = std::mem::transmute(crate::utils::get_proc_address(_context, "glGenVertexArrays"));
            self.glGenVertexArraysOES = std::mem::transmute(crate::utils::get_proc_address(_context, "glGenVertexArraysOES"));
            self.glGenerateMipmap = std::mem::transmute(crate::utils::get_proc_address(_context, "glGenerateMipmap"));
            self.glGetActiveAttrib = std::mem::transmute(crate::utils::get_proc_address(_context, "glGetActiveAttrib"));
            self.glGetActiveUniform = std::mem::transmute(crate::utils::get_proc_address(_context, "glGetActiveUniform"));
            self.glGetActiveUniformBlockName = std::mem::transmute(crate::utils::get_proc_address(_context, "glGetActiveUniformBlockName"));
            self.glGetActiveUniformBlockiv = std::mem::transmute(crate::utils::get_proc_address(_context, "glGetActiveUniformBlockiv"));
            self.glGetActiveUniformsiv = std::mem::transmute(crate::utils::get_proc_address(_context, "glGetActiveUniformsiv"));
            self.glGetAttachedShaders = std::mem::transmute(crate::utils::get_proc_address(_context, "glGetAttachedShaders"));
            self.glGetAttribLocation = std::mem::transmute(crate::utils::get_proc_address(_context, "glGetAttribLocation"));
            self.glGetBooleanv = std::mem::transmute(crate::utils::get_proc_address(_context, "glGetBooleanv"));
            self.glGetBufferParameteri64v = std::mem::transmute(crate::utils::get_proc_address(_context, "glGetBufferParameteri64v"));
            self.glGetBufferParameteriv = std::mem::transmute(crate::utils::get_proc_address(_context, "glGetBufferParameteriv"));
            self.glGetBufferPointerv = std::mem::transmute(crate::utils::get_proc_address(_context, "glGetBufferPointerv"));
            self.glGetError = std::mem::transmute(crate::utils::get_proc_address(_context, "glGetError"));
            self.glGetFloatv = std::mem::transmute(crate::utils::get_proc_address(_context, "glGetFloatv"));
            self.glGetFragDataLocation = std::mem::transmute(crate::utils::get_proc_address(_context, "glGetFragDataLocation"));
            self.glGetFramebufferAttachmentParameteriv = std::mem::transmute(crate::utils::get_proc_address(_context, "glGetFramebufferAttachmentParameteriv"));
            self.glGetInteger64i_v = std::mem::transmute(crate::utils::get_proc_address(_context, "glGetInteger64i_v"));
            self.glGetInteger64v = std::mem::transmute(crate::utils::get_proc_address(_context, "glGetInteger64v"));
            self.glGetIntegeri_v = std::mem::transmute(crate::utils::get_proc_address(_context, "glGetIntegeri_v"));
            self.glGetIntegerv = std::mem::transmute(crate::utils::get_proc_address(_context, "glGetIntegerv"));
            self.glGetInternalformativ = std::mem::transmute(crate::utils::get_proc_address(_context, "glGetInternalformativ"));
            self.glGetProgramBinary = std::mem::transmute(crate::utils::get_proc_address(_context, "glGetProgramBinary"));
            self.glGetProgramInfoLog = std::mem::transmute(crate::utils::get_proc_address(_context, "glGetProgramInfoLog"));
            self.glGetProgramiv = std::mem::transmute(crate::utils::get_proc_address(_context, "glGetProgramiv"));
            self.glGetQueryObjectuiv = std::mem::transmute(crate::utils::get_proc_address(_context, "glGetQueryObjectuiv"));
            self.glGetQueryiv = std::mem::transmute(crate::utils::get_proc_address(_context, "glGetQueryiv"));
            self.glGetRenderbufferParameteriv = std::mem::transmute(crate::utils::get_proc_address(_context, "glGetRenderbufferParameteriv"));
            self.glGetSamplerParameterfv = std::mem::transmute(crate::utils::get_proc_address(_context, "glGetSamplerParameterfv"));
            self.glGetSamplerParameteriv = std::mem::transmute(crate::utils::get_proc_address(_context, "glGetSamplerParameteriv"));
            self.glGetShaderInfoLog = std::mem::transmute(crate::utils::get_proc_address(_context, "glGetShaderInfoLog"));
            self.glGetShaderPrecisionFormat = std::mem::transmute(crate::utils::get_proc_address(_context, "glGetShaderPrecisionFormat"));
            self.glGetShaderSource = std::mem::transmute(crate::utils::get_proc_address(_context, "glGetShaderSource"));
            self.glGetShaderiv = std::mem::transmute(crate::utils::get_proc_address(_context, "glGetShaderiv"));
            self.glGetString = std::mem::transmute(crate::utils::get_proc_address(_context, "glGetString"));
            self.glGetStringi = std::mem::transmute(crate::utils::get_proc_address(_context, "glGetStringi"));
            self.glGetSynciv = std::mem::transmute(crate::utils::get_proc_address(_context, "glGetSynciv"));
            self.glGetTexParameterfv = std::mem::transmute(crate::utils::get_proc_address(_context, "glGetTexParameterfv"));
            self.glGetTexParameteriv = std::mem::transmute(crate::utils::get_proc_address(_context, "glGetTexParameteriv"));
            self.glGetTransformFeedbackVarying = std::mem::transmute(crate::utils::get_proc_address(_context, "glGetTransformFeedbackVarying"));
            self.glGetUniformBlockIndex = std::mem::transmute(crate::utils::get_proc_address(_context, "glGetUniformBlockIndex"));
            self.glGetUniformIndices = std::mem::transmute(crate::utils::get_proc_address(_context, "glGetUniformIndices"));
            self.glGetUniformLocation = std::mem::transmute(crate::utils::get_proc_address(_context, "glGetUniformLocation"));
            self.glGetUniformfv = std::mem::transmute(crate::utils::get_proc_address(_context, "glGetUniformfv"));
            self.glGetUniformiv = std::mem::transmute(crate::utils::get_proc_address(_context, "glGetUniformiv"));
            self.glGetUniformuiv = std::mem::transmute(crate::utils::get_proc_address(_context, "glGetUniformuiv"));
            self.glGetVertexAttribIiv = std::mem::transmute(crate::utils::get_proc_address(_context, "glGetVertexAttribIiv"));
            self.glGetVertexAttribIuiv = std::mem::transmute(crate::utils::get_proc_address(_context, "glGetVertexAttribIuiv"));
            self.glGetVertexAttribPointerv = std::mem::transmute(crate::utils::get_proc_address(_context, "glGetVertexAttribPointerv"));
            self.glGetVertexAttribfv = std::mem::transmute(crate::utils::get_proc_address(_context, "glGetVertexAttribfv"));
            self.glGetVertexAttribiv = std::mem::transmute(crate::utils::get_proc_address(_context, "glGetVertexAttribiv"));
            self.glHint = std::mem::transmute(crate::utils::get_proc_address(_context, "glHint"));
            self.glInvalidateFramebuffer = std::mem::transmute(crate::utils::get_proc_address(_context, "glInvalidateFramebuffer"));
            self.glInvalidateSubFramebuffer = std::mem::transmute(crate::utils::get_proc_address(_context, "glInvalidateSubFramebuffer"));
            self.glIsBuffer = std::mem::transmute(crate::utils::get_proc_address(_context, "glIsBuffer"));
            self.glIsEnabled = std::mem::transmute(crate::utils::get_proc_address(_context, "glIsEnabled"));
            self.glIsFramebuffer = std::mem::transmute(crate::utils::get_proc_address(_context, "glIsFramebuffer"));
            self.glIsProgram = std::mem::transmute(crate::utils::get_proc_address(_context, "glIsProgram"));
            self.glIsQuery = std::mem::transmute(crate::utils::get_proc_address(_context, "glIsQuery"));
            self.glIsRenderbuffer = std::mem::transmute(crate::utils::get_proc_address(_context, "glIsRenderbuffer"));
            self.glIsSampler = std::mem::transmute(crate::utils::get_proc_address(_context, "glIsSampler"));
            self.glIsShader = std::mem::transmute(crate::utils::get_proc_address(_context, "glIsShader"));
            self.glIsSync = std::mem::transmute(crate::utils::get_proc_address(_context, "glIsSync"));
            self.glIsTexture = std::mem::transmute(crate::utils::get_proc_address(_context, "glIsTexture"));
            self.glIsTransformFeedback = std::mem::transmute(crate::utils::get_proc_address(_context, "glIsTransformFeedback"));
            self.glIsVertexArray = std::mem::transmute(crate::utils::get_proc_address(_context, "glIsVertexArray"));
            self.glIsVertexArrayOES = std::mem::transmute(crate::utils::get_proc_address(_context, "glIsVertexArrayOES"));
            self.glLineWidth = std::mem::transmute(crate::utils::get_proc_address(_context, "glLineWidth"));
            self.glLinkProgram = std::mem::transmute(crate::utils::get_proc_address(_context, "glLinkProgram"));
            self.glMapBufferRange = std::mem::transmute(crate::utils::get_proc_address(_context, "glMapBufferRange"));
            self.glPauseTransformFeedback = std::mem::transmute(crate::utils::get_proc_address(_context, "glPauseTransformFeedback"));
            self.glPixelStorei = std::mem::transmute(crate::utils::get_proc_address(_context, "glPixelStorei"));
            self.glPolygonOffset = std::mem::transmute(crate::utils::get_proc_address(_context, "glPolygonOffset"));
            self.glProgramBinary = std::mem::transmute(crate::utils::get_proc_address(_context, "glProgramBinary"));
            self.glProgramParameteri = std::mem::transmute(crate::utils::get_proc_address(_context, "glProgramParameteri"));
            self.glReadBuffer = std::mem::transmute(crate::utils::get_proc_address(_context, "glReadBuffer"));
            self.glReadPixels = std::mem::transmute(crate::utils::get_proc_address(_context, "glReadPixels"));
            self.glReleaseShaderCompiler = std::mem::transmute(crate::utils::get_proc_address(_context, "glReleaseShaderCompiler"));
            self.glRenderbufferStorage = std::mem::transmute(crate::utils::get_proc_address(_context, "glRenderbufferStorage"));
            self.glRenderbufferStorageMultisample = std::mem::transmute(crate::utils::get_proc_address(_context, "glRenderbufferStorageMultisample"));
            self.glResumeTransformFeedback = std::mem::transmute(crate::utils::get_proc_address(_context, "glResumeTransformFeedback"));
            self.glSampleCoverage = std::mem::transmute(crate::utils::get_proc_address(_context, "glSampleCoverage"));
            self.glSamplerParameterf = std::mem::transmute(crate::utils::get_proc_address(_context, "glSamplerParameterf"));
            self.glSamplerParameterfv = std::mem::transmute(crate::utils::get_proc_address(_context, "glSamplerParameterfv"));
            self.glSamplerParameteri = std::mem::transmute(crate::utils::get_proc_address(_context, "glSamplerParameteri"));
            self.glSamplerParameteriv = std::mem::transmute(crate::utils::get_proc_address(_context, "glSamplerParameteriv"));
            self.glScissor = std::mem::transmute(crate::utils::get_proc_address(_context, "glScissor"));
            self.glShaderBinary = std::mem::transmute(crate::utils::get_proc_address(_context, "glShaderBinary"));
            self.glShaderSource = std::mem::transmute(crate::utils::get_proc_address(_context, "glShaderSource"));
            self.glStencilFunc = std::mem::transmute(crate::utils::get_proc_address(_context, "glStencilFunc"));
            self.glStencilFuncSeparate = std::mem::transmute(crate::utils::get_proc_address(_context, "glStencilFuncSeparate"));
            self.glStencilMask = std::mem::transmute(crate::utils::get_proc_address(_context, "glStencilMask"));
            self.glStencilMaskSeparate = std::mem::transmute(crate::utils::get_proc_address(_context, "glStencilMaskSeparate"));
            self.glStencilOp = std::mem::transmute(crate::utils::get_proc_address(_context, "glStencilOp"));
            self.glStencilOpSeparate = std::mem::transmute(crate::utils::get_proc_address(_context, "glStencilOpSeparate"));
            self.glTexImage2D = std::mem::transmute(crate::utils::get_proc_address(_context, "glTexImage2D"));
            self.glTexImage3D = std::mem::transmute(crate::utils::get_proc_address(_context, "glTexImage3D"));
            self.glTexParameterf = std::mem::transmute(crate::utils::get_proc_address(_context, "glTexParameterf"));
            self.glTexParameterfv = std::mem::transmute(crate::utils::get_proc_address(_context, "glTexParameterfv"));
            self.glTexParameteri = std::mem::transmute(crate::utils::get_proc_address(_context, "glTexParameteri"));
            self.glTexParameteriv = std::mem::transmute(crate::utils::get_proc_address(_context, "glTexParameteriv"));
            self.glTexStorage2D = std::mem::transmute(crate::utils::get_proc_address(_context, "glTexStorage2D"));
            self.glTexStorage3D = std::mem::transmute(crate::utils::get_proc_address(_context, "glTexStorage3D"));
            self.glTexSubImage2D = std::mem::transmute(crate::utils::get_proc_address(_context, "glTexSubImage2D"));
            self.glTexSubImage3D = std::mem::transmute(crate::utils::get_proc_address(_context, "glTexSubImage3D"));
            self.glTransformFeedbackVaryings = std::mem::transmute(crate::utils::get_proc_address(_context, "glTransformFeedbackVaryings"));
            self.glUniform1f = std::mem::transmute(crate::utils::get_proc_address(_context, "glUniform1f"));
            self.glUniform1fv = std::mem::transmute(crate::utils::get_proc_address(_context, "glUniform1fv"));
            self.glUniform1i = std::mem::transmute(crate::utils::get_proc_address(_context, "glUniform1i"));
            self.glUniform1iv = std::mem::transmute(crate::utils::get_proc_address(_context, "glUniform1iv"));
            self.glUniform1ui = std::mem::transmute(crate::utils::get_proc_address(_context, "glUniform1ui"));
            self.glUniform1uiv = std::mem::transmute(crate::utils::get_proc_address(_context, "glUniform1uiv"));
            self.glUniform2f = std::mem::transmute(crate::utils::get_proc_address(_context, "glUniform2f"));
            self.glUniform2fv = std::mem::transmute(crate::utils::get_proc_address(_context, "glUniform2fv"));
            self.glUniform2i = std::mem::transmute(crate::utils::get_proc_address(_context, "glUniform2i"));
            self.glUniform2iv = std::mem::transmute(crate::utils::get_proc_address(_context, "glUniform2iv"));
            self.glUniform2ui = std::mem::transmute(crate::utils::get_proc_address(_context, "glUniform2ui"));
            self.glUniform2uiv = std::mem::transmute(crate::utils::get_proc_address(_context, "glUniform2uiv"));
            self.glUniform3f = std::mem::transmute(crate::utils::get_proc_address(_context, "glUniform3f"));
            self.glUniform3fv = std::mem::transmute(crate::utils::get_proc_address(_context, "glUniform3fv"));
            self.glUniform3i = std::mem::transmute(crate::utils::get_proc_address(_context, "glUniform3i"));
            self.glUniform3iv = std::mem::transmute(crate::utils::get_proc_address(_context, "glUniform3iv"));
            self.glUniform3ui = std::mem::transmute(crate::utils::get_proc_address(_context, "glUniform3ui"));
            self.glUniform3uiv = std::mem::transmute(crate::utils::get_proc_address(_context, "glUniform3uiv"));
            self.glUniform4f = std::mem::transmute(crate::utils::get_proc_address(_context, "glUniform4f"));
            self.glUniform4fv = std::mem::transmute(crate::utils::get_proc_address(_context, "glUniform4fv"));
            self.glUniform4i = std::mem::transmute(crate::utils::get_proc_address(_context, "glUniform4i"));
            self.glUniform4iv = std::mem::transmute(crate::utils::get_proc_address(_context, "glUniform4iv"));
            self.glUniform4ui = std::mem::transmute(crate::utils::get_proc_address(_context, "glUniform4ui"));
            self.glUniform4uiv = std::mem::transmute(crate::utils::get_proc_address(_context, "glUniform4uiv"));
            self.glUniformBlockBinding = std::mem::transmute(crate::utils::get_proc_address(_context, "glUniformBlockBinding"));
            self.glUniformMatrix2fv = std::mem::transmute(crate::utils::get_proc_address(_context, "glUniformMatrix2fv"));
            self.glUniformMatrix2x3fv = std::mem::transmute(crate::utils::get_proc_address(_context, "glUniformMatrix2x3fv"));
            self.glUniformMatrix2x4fv = std::mem::transmute(crate::utils::get_proc_address(_context, "glUniformMatrix2x4fv"));
            self.glUniformMatrix3fv = std::mem::transmute(crate::utils::get_proc_address(_context, "glUniformMatrix3fv"));
            self.glUniformMatrix3x2fv = std::mem::transmute(crate::utils::get_proc_address(_context, "glUniformMatrix3x2fv"));
            self.glUniformMatrix3x4fv = std::mem::transmute(crate::utils::get_proc_address(_context, "glUniformMatrix3x4fv"));
            self.glUniformMatrix4fv = std::mem::transmute(crate::utils::get_proc_address(_context, "glUniformMatrix4fv"));
            self.glUniformMatrix4x2fv = std::mem::transmute(crate::utils::get_proc_address(_context, "glUniformMatrix4x2fv"));
            self.glUniformMatrix4x3fv = std::mem::transmute(crate::utils::get_proc_address(_context, "glUniformMatrix4x3fv"));
            self.glUnmapBuffer = std::mem::transmute(crate::utils::get_proc_address(_context, "glUnmapBuffer"));
            self.glUseProgram = std::mem::transmute(crate::utils::get_proc_address(_context, "glUseProgram"));
            self.glValidateProgram = std::mem::transmute(crate::utils::get_proc_address(_context, "glValidateProgram"));
            self.glVertexAttrib1f = std::mem::transmute(crate::utils::get_proc_address(_context, "glVertexAttrib1f"));
            self.glVertexAttrib1fv = std::mem::transmute(crate::utils::get_proc_address(_context, "glVertexAttrib1fv"));
            self.glVertexAttrib2f = std::mem::transmute(crate::utils::get_proc_address(_context, "glVertexAttrib2f"));
            self.glVertexAttrib2fv = std::mem::transmute(crate::utils::get_proc_address(_context, "glVertexAttrib2fv"));
            self.glVertexAttrib3f = std::mem::transmute(crate::utils::get_proc_address(_context, "glVertexAttrib3f"));
            self.glVertexAttrib3fv = std::mem::transmute(crate::utils::get_proc_address(_context, "glVertexAttrib3fv"));
            self.glVertexAttrib4f = std::mem::transmute(crate::utils::get_proc_address(_context, "glVertexAttrib4f"));
            self.glVertexAttrib4fv = std::mem::transmute(crate::utils::get_proc_address(_context, "glVertexAttrib4fv"));
            self.glVertexAttribDivisor = std::mem::transmute(crate::utils::get_proc_address(_context, "glVertexAttribDivisor"));
            self.glVertexAttribI4i = std::mem::transmute(crate::utils::get_proc_address(_context, "glVertexAttribI4i"));
            self.glVertexAttribI4iv = std::mem::transmute(crate::utils::get_proc_address(_context, "glVertexAttribI4iv"));
            self.glVertexAttribI4ui = std::mem::transmute(crate::utils::get_proc_address(_context, "glVertexAttribI4ui"));
            self.glVertexAttribI4uiv = std::mem::transmute(crate::utils::get_proc_address(_context, "glVertexAttribI4uiv"));
            self.glVertexAttribIPointer = std::mem::transmute(crate::utils::get_proc_address(_context, "glVertexAttribIPointer"));
            self.glVertexAttribPointer = std::mem::transmute(crate::utils::get_proc_address(_context, "glVertexAttribPointer"));
            self.glViewport = std::mem::transmute(crate::utils::get_proc_address(_context, "glViewport"));
            self.glWaitSync = std::mem::transmute(crate::utils::get_proc_address(_context, "glWaitSync"));
        }
    }
}
