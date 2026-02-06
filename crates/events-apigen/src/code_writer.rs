use std::{
    fs::File,
    io::Read as _,
    sync::{atomic::AtomicUsize, Arc},
};

const BEGIN_GENERATED_CODE_SEP: &str = "// BEGIN GENERATED CODE";
const END_GENERATED_CODE_SEP: &str = "// END GENERATED CODE";

pub struct CodeWriter {
    buf: Vec<u8>,
    after_generated: Vec<u8>,
    indent_storage: IndentStorage,
    original_content: Vec<u8>,
    path: std::path::PathBuf,
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
    pub fn new(path: std::path::PathBuf) -> anyhow::Result<Self> {
        let mut original_content = vec![];
        File::open(&path)
            .unwrap()
            .read_to_end(&mut original_content)
            .unwrap();

        let original_content_string = String::from_utf8(original_content.clone())?;
        let (before_generated, the_rest) = original_content_string
            .split_once(BEGIN_GENERATED_CODE_SEP)
            .unwrap();
        let (_, after_generated) = the_rest.split_once(END_GENERATED_CODE_SEP).unwrap();

        Ok(CodeWriter {
            buf: (before_generated.to_owned() + BEGIN_GENERATED_CODE_SEP + "\n")
                .as_bytes()
                .to_vec(),
            after_generated: (END_GENERATED_CODE_SEP.to_owned() + after_generated)
                .as_bytes()
                .to_vec(),
            indent_storage: IndentStorage {
                indent: Arc::new(AtomicUsize::new(0)),
            },
            original_content,
            path,
        })
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

    pub fn write_to_file(mut self) -> anyhow::Result<()> {
        let mut content = self.buf;
        content.append(&mut self.after_generated);
        if content == self.original_content {
            return Ok(());
        }
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(self.path)?;
        std::io::Write::write_all(&mut file, &content)?;
        Ok(())
    }
}
