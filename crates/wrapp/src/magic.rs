use webrogue_common::RandomAccessReader;

pub const MAGIC: &'static [u8; 6] = b"WRAPP\0";

pub fn is_a_wrapp(readable: &mut impl RandomAccessReader) -> anyhow::Result<bool> {
    let mut magic = [0u8; MAGIC.len()];
    readable.seek(std::io::SeekFrom::Start(0))?;
    if readable
        .seek(std::io::SeekFrom::Start(MAGIC.len() as u64))
        .is_err()
    {
        return Ok(false);
    };
    readable.seek(std::io::SeekFrom::Start(0))?;
    readable.read_exact(&mut magic)?;
    Ok(magic == *MAGIC)
}

pub fn is_path_a_wrapp<P: AsRef<std::path::Path>>(path: P) -> anyhow::Result<bool> {
    let mut file = std::fs::File::open(path)?;
    is_a_wrapp(&mut file)
}
