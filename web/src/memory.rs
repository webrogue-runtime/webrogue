use wasm_bindgen::JsCast as _;
use web_sys::js_sys::{ArrayBuffer, SharedArrayBuffer, Uint8Array, WebAssembly};

#[derive(Clone)]
pub struct Memory {
    memory_object: WebAssembly::Memory,
    is_shared: bool,
}

impl Memory {
    pub fn new(memory_object: WebAssembly::Memory) -> Self {
        let is_memory_shared = memory_object.buffer().is_instance_of::<SharedArrayBuffer>();
        Memory {
            memory_object,
            is_shared: is_memory_shared,
        }
    }
    fn as_typed_array(&self) -> Uint8Array {
        let buffer = self.memory_object.buffer();
        if self.is_shared {
            Uint8Array::new(&buffer.dyn_into::<SharedArrayBuffer>().unwrap())
        } else {
            Uint8Array::new(&buffer.dyn_into::<ArrayBuffer>().unwrap())
        }
    }
}

impl wiggle::DynamicGuestMemory for Memory {
    fn size(&self) -> usize {
        self.as_typed_array().length() as usize
    }

    fn write(&mut self, offset: u32, data: &[u8]) {
        self.as_typed_array()
            .subarray(offset, offset + data.len() as u32)
            .copy_from(data);
    }

    fn read(&self, offset: u32, data: &mut [u8]) {
        self.as_typed_array()
            .subarray(offset, offset + data.len() as u32)
            .copy_to(data);
    }
}
