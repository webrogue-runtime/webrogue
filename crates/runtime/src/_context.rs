use std::{ffi::c_void, ptr::null_mut};

pub struct ContextVec {
    contexts: Vec<*mut c_void>,
}

impl ContextVec {
    pub fn new() -> Self {
        Self { contexts: vec![] }
    }

    pub unsafe fn get<Context>(&self, i: usize) -> &mut Context {
        let ptr = self.contexts[i] as *mut Context;
        return unsafe { &mut *ptr };
    }

    pub fn get_raw<Context>(&self, i: usize) -> *mut Context {
        self.contexts[i] as *mut Context
    }

    pub fn set<Context>(&mut self, i: usize, ptr: *mut Context) {
        while self.contexts.len() < i + 1 {
            self.contexts.push(null_mut());
        }
        self.contexts[i] = ptr as *mut c_void;
    }
}
