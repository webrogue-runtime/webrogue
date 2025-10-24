mod link;

use std::io::{Seek, Write};

use anyhow::Context as _;

#[derive(Clone, Debug)]
pub enum LibC {
    GLibC,
    MUSL,
}
impl clap::ValueEnum for LibC {
    fn value_variants<'a>() -> &'a [Self] {
        &[Self::GLibC, Self::MUSL]
    }

    fn to_possible_value(&self) -> Option<clap::builder::PossibleValue> {
        match self {
            Self::GLibC => Some(clap::builder::PossibleValue::new("glibc")),
            Self::MUSL => Some(clap::builder::PossibleValue::new("musl")),
        }
    }
}

pub fn build_linux(
    wrapp_file_path: &std::path::PathBuf,
    output_file_path: &std::path::PathBuf,
    libc: LibC,
    cache: Option<&std::path::PathBuf>,
) -> anyhow::Result<()> {
    let object_file = crate::utils::TemporalFile::for_tmp_object(output_file_path)?;
    match libc {
        LibC::GLibC => {
            println!("Compiling AOT object...");
            crate::compile::compile_wrapp_to_object(
                wrapp_file_path,
                object_file.path(),
                crate::Target::X86_64LinuxGNU,
                cache,
                false,
                false,
            )?;

            println!("Linking native binary...");
            link::link_glibc(&object_file, output_file_path)?;
        }
        LibC::MUSL => {
            println!("Compiling AOT object...");
            crate::compile::compile_wrapp_to_object(
                wrapp_file_path,
                object_file.path(),
                crate::Target::X86_64LinuxMUSL,
                cache,
                true,
                false,
            )?;

            println!("Linking native binary...");
            link::link_musl(&object_file, output_file_path)?;
        }
    }

    drop(object_file);

    println!("Embedding stripped WRAPP file...");
    let mut output_file: std::fs::File = std::fs::OpenOptions::new()
        .append(true)
        .create(false)
        .open(output_file_path)?;

    let original_size = output_file.seek(std::io::SeekFrom::End(0))?;

    if webrogue_wrapp::is_path_a_wrapp(&wrapp_file_path).with_context(|| {
        format!(
            "Unable to determine file type for {}",
            wrapp_file_path.display()
        )
    })? {
        webrogue_wrapp::WRAPPWriter::new(webrogue_wrapp::WrappVFSBuilder::from_file_path(
            wrapp_file_path,
        )?)
        .write(&mut output_file)?;
    } else {
        webrogue_wrapp::WRAPPWriter::new(webrogue_wrapp::RealVFSBuilder::new(wrapp_file_path)?)
            .write(&mut output_file)?;
    }
    let new_size = output_file.seek(std::io::SeekFrom::End(0))?;

    let wrapp_size = new_size - original_size;
    output_file.write_all(&(wrapp_size as u64).to_le_bytes())?;

    anyhow::Ok(())
}
