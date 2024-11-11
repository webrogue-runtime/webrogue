pub fn guest_strlen(memory: &crate::memory_handle::MemoryHandle, ptr: u64) -> usize {
    let mut new_ptr = ptr;
    loop {
        let read_result = memory.read::<u8>(new_ptr);
        match read_result {
            Ok(byte) => {
                if byte == 0 {
                    break;
                }
            }
            Err(_) => break,
        }
        new_ptr += 1
    }
    return (new_ptr - ptr) as usize;
}
