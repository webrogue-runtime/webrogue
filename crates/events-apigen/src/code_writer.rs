pub struct CodeWriter {
    buf: Vec<u8>,
    indent: usize,
}

impl CodeWriter {
    pub fn new() -> Self {
        CodeWriter {
            buf: Vec::new(),
            indent: 0,
        }
    }

    pub fn inc_indent(&mut self) {
        self.indent += 1;
    }

    pub fn dec_indent(&mut self) {
        self.indent -= 1;
    }

    pub fn writeln(&mut self, s: &str) -> anyhow::Result<()> {
        let new_string = " ".repeat(self.indent * 4) + s + "\n";
        std::io::Write::write_all(&mut self.buf, new_string.as_bytes())?;
        anyhow::Ok(())
    }

    pub fn write_to_file(&mut self, path: &std::path::Path) -> anyhow::Result<()> {
        let mut file = std::fs::OpenOptions::new().read(true).open(path)?;
        let mut old_content = Vec::new();
        std::io::Read::read_to_end(&mut file, &mut old_content)?;
        if old_content == self.buf {
            return Ok(());
        }
        drop(file);
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(path)?;
        std::io::Write::write_all(&mut file, &self.buf)?;
        Ok(())
    }
}