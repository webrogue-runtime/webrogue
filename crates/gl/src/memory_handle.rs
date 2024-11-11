use std::mem::MaybeUninit;
use std::result::Result::Ok;
use wasmer::{AsStoreRef, MemoryAccessError};

pub struct MemoryHandle<'a> {
    memory: wasmer::MemoryView<'a>,
}

impl<'a> MemoryHandle<'a> {
    pub fn new(memory: wasmer::Memory, store: &'a (impl AsStoreRef + ?Sized)) -> Self {
        Self {
            memory: memory.view(store),
        }
    }

    pub fn read<T: WasmType>(&self, offset: u64) -> std::result::Result<T, MemoryAccessError> {
        T::read(self, offset)
    }

    pub fn read_vec<T: WasmType>(
        &self,
        offset: u64,
        count: u64,
    ) -> std::result::Result<Vec<T>, MemoryAccessError> {
        let mut result: Vec<T> = vec![T::ZERO; count as usize];
        T::read_slice(self, offset, result.as_mut_slice())?;
        Ok(result)
    }

    // pub fn read_slice<T: WasmType>(
    //     &self,
    //     offset: u64,
    //     buffer: &mut [T],
    // ) -> std::result::Result<(), MemoryAccessError> {
    //     T::read_slice(self, offset, buffer)
    // }

    // pub fn write<T: WasmType>(
    //     &self,
    //     offset: u64,
    //     value: T,
    // ) -> std::result::Result<(), MemoryAccessError> {
    //     T::write(self, offset, value)
    // }

    pub fn write_slice<T: WasmType>(
        &self,
        offset: u64,
        buffer: &[T],
    ) -> std::result::Result<(), MemoryAccessError> {
        T::write_slice(self, offset, buffer)
    }
}

pub trait WasmType: Sized + Clone + Copy {
    fn read(mh: &MemoryHandle, offset: u64) -> std::result::Result<Self, MemoryAccessError>;
    fn read_slice(
        mh: &MemoryHandle,
        offset: u64,
        buffer: &mut [Self],
    ) -> std::result::Result<(), MemoryAccessError>;
    // fn write(
    //     mh: &MemoryHandle,
    //     offset: u64,
    //     value: Self,
    // ) -> std::result::Result<(), MemoryAccessError>;
    fn write_slice(
        mh: &MemoryHandle,
        offset: u64,
        buffer: &[Self],
    ) -> std::result::Result<(), MemoryAccessError>;
    const ZERO: Self;
}

macro_rules! opaque_le_type {
    ($ty:ident, width: $width:literal, zero: $zero:literal) => {
        impl WasmType for $ty {
            fn read(
                mh: &MemoryHandle,
                offset: u64,
            ) -> std::result::Result<Self, MemoryAccessError> {
                let mut buffer: MaybeUninit<[u8; $width]> = MaybeUninit::uninit();
                mh.memory
                    .read(offset, unsafe { buffer.assume_init_mut() })?;
                Ok(Self::from_le_bytes(unsafe { buffer.assume_init_read() }))
            }

            fn read_slice(
                mh: &MemoryHandle,
                offset: u64,
                buffer: &mut [Self],
            ) -> std::result::Result<(), MemoryAccessError> {
                mh.memory.read(offset, unsafe {
                    std::slice::from_raw_parts_mut(
                        buffer.as_mut_ptr() as *mut u8,
                        buffer.len() * $width,
                    )
                })?;
                Ok(())
            }

            // fn write(
            //     mh: &MemoryHandle,
            //     offset: u64,
            //     value: Self,
            // ) -> std::result::Result<(), MemoryAccessError> {
            //     mh.memory.write(offset, &Self::to_le_bytes(value))?;
            //     Ok(())
            // }

            fn write_slice(
                mh: &MemoryHandle,
                offset: u64,
                buffer: &[Self],
            ) -> std::result::Result<(), MemoryAccessError> {
                mh.memory.write(offset, unsafe {
                    std::slice::from_raw_parts(buffer.as_ptr() as *mut u8, buffer.len() * $width)
                })?;
                Ok(())
            }

            const ZERO: Self = $zero;
        }
    };
}

opaque_le_type! {u8, width: 1, zero: 0}
opaque_le_type! {u16, width: 2, zero: 0}
opaque_le_type! {u32, width: 4, zero: 0}
opaque_le_type! {u64, width: 8, zero: 0}
opaque_le_type! {i8, width: 1, zero: 0}
opaque_le_type! {i16, width: 2, zero: 0}
opaque_le_type! {i32, width: 4, zero: 0}
opaque_le_type! {i64, width: 8, zero: 0}
opaque_le_type! {f32, width: 4, zero: 0.0}
opaque_le_type! {f64, width: 8, zero: 0.0}
