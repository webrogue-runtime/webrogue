// TODO glMapBufferRange

pub const EXCLUDED: [&str; 2] = ["glGetString", "glGetStringi"];

pub const MANUAL_IMPL: [&str; 10] = [
    "glShaderSource",
    "glVertexAttribPointer",
    "glVertexAttribIPointer",
    "glTexImage2D",
    "glBufferSubData",
    "glDrawElements",
    "glDrawElementsInstanced",
    "glDrawRangeElements",
    "glGetUniformIndices",
    "glMapBufferRange", // not implemented
];

pub const C_MANUAL_IMPL: [&str; 0] = [];

pub const EXTRA_EXTENSIONS: [&str; 1] = ["GL_OES_vertex_array_object"];
