use std::sync::{atomic::AtomicUsize, Arc};

pub struct CodeWriter {
    buf: Vec<u8>,
    indent_storage: IndentStorage,
}

#[derive(Clone)]
pub struct IndentStorage {
    indent: Arc<AtomicUsize>,
}

impl IndentStorage {
    pub fn inc(&self, f: impl FnOnce()) {
        self.indent
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        f();
        self.indent
            .fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
    }
}

impl CodeWriter {
    pub fn new() -> Self {
        CodeWriter {
            buf: Vec::new(),
            indent_storage: IndentStorage {
                indent: Arc::new(AtomicUsize::new(0)),
            },
        }
    }

    pub fn indent_storage(&self) -> IndentStorage {
        self.indent_storage.clone()
    }

    pub fn writeln(&mut self, s: &str) -> anyhow::Result<()> {
        let new_string = " ".repeat(
            self.indent_storage
                .indent
                .load(std::sync::atomic::Ordering::SeqCst)
                * 4,
        ) + s
            + "\n";
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
