#[allow(non_camel_case_types)]
#[derive(Clone, Copy)]
pub enum Target {
    X86_64LinuxGNU,
    x86_64WindowsGNU,
    x86_64WindowsMSVC,
    x86_64AppleDarwin,
    ARM64AppleDarwin,
    X86_64AppleIOSSIM,
    ARM64AppleIOSSIM,
    ARM64AppleIOS,
    ARM64LinuxAndroid,
    // TODO
    // aarch64-linux-gnu
}

impl Target {
    pub fn from_name(name: &str) -> anyhow::Result<Self> {
        Self::all_cases()
            .find(|target| target.name() == name)
            .and_then(|target| Some(*target))
            .ok_or_else(|| {
                anyhow::anyhow!("Target {} is not supported by webrogue-aot-compiler", name)
            })
    }
    pub fn all_cases() -> std::slice::Iter<'static, Target> {
        use Target::*;
        [
            X86_64LinuxGNU,
            x86_64WindowsGNU,
            x86_64WindowsMSVC,
            x86_64AppleDarwin,
            ARM64AppleDarwin,
            X86_64AppleIOSSIM,
            ARM64AppleIOSSIM,
            ARM64AppleIOS,
            ARM64LinuxAndroid,
        ]
        .iter()
    }

    pub fn name(&self) -> &'static str {
        use Target::*;
        match self {
            X86_64LinuxGNU => "x86_64-linux-gnu",
            x86_64WindowsGNU => "x86_64-windows-gnu",
            x86_64WindowsMSVC => "x86_64-windows-msvc",
            x86_64AppleDarwin => "x86_64-apple-darwin",
            ARM64AppleDarwin => "arm64-apple-darwin",
            X86_64AppleIOSSIM => "x86_64-apple-ios",
            ARM64AppleIOSSIM => "arm64-apple-ios-sim",
            ARM64AppleIOS => "arm64-apple-ios",
            ARM64LinuxAndroid => "aarch64-linux-android",
        }
    }
}

impl Target {
    fn object_info(
        &self,
    ) -> (
        object::BinaryFormat,
        object::Architecture,
        object::Endianness,
    ) {
        use object::Architecture::*;
        use object::BinaryFormat::*;
        use object::Endianness::*;
        use Target::*;
        match self {
            X86_64LinuxGNU => (Elf, X86_64, Little),
            ARM64LinuxAndroid => (Elf, Aarch64, Little),
            x86_64WindowsGNU => (Coff, X86_64, Little),
            x86_64WindowsMSVC => (Coff, X86_64, Little),
            x86_64AppleDarwin => (MachO, X86_64, Little),
            ARM64AppleDarwin => (MachO, Aarch64, Little),
            X86_64AppleIOSSIM => (MachO, X86_64, Little),
            ARM64AppleIOSSIM => (MachO, Aarch64, Little),
            ARM64AppleIOS => (MachO, Aarch64, Little),
            // _ => unimplemented!("{}", self.name()),
        }
    }

    pub fn format(&self) -> object::BinaryFormat {
        self.object_info().0
    }

    pub fn arch(&self) -> object::Architecture {
        self.object_info().1
    }

    pub fn endianness(&self) -> object::Endianness {
        self.object_info().2
    }
}

impl Target {
    fn object_reloc(
        &self,
        is_pic: bool,
    ) -> (i64, object::RelocationKind, object::RelocationEncoding, u8) {
        use object::RelocationEncoding::*;
        use object::RelocationKind::*;
        use Target::*;
        match (self, is_pic) {
            (X86_64LinuxGNU | x86_64AppleDarwin | x86_64WindowsGNU | x86_64WindowsMSVC, false) => {
                (-4, Relative, Generic, 32)
            }
            (
                X86_64LinuxGNU | x86_64AppleDarwin | X86_64AppleIOSSIM | ARM64AppleDarwin
                | ARM64AppleIOSSIM | ARM64AppleIOS | x86_64WindowsGNU | x86_64WindowsMSVC | ARM64LinuxAndroid,
                true,
            ) => (-4, GotRelative, Generic, 32),
            _ => unimplemented!("target: {}, is_pic: {}", self.name(), is_pic),
        }
    }

    pub fn reloc_append(&self, is_pic: bool) -> i64 {
        self.object_reloc(is_pic).0
    }

    pub fn reloc_kind(&self, is_pic: bool) -> object::RelocationKind {
        self.object_reloc(is_pic).1
    }

    pub fn reloc_encoding(&self, is_pic: bool) -> object::RelocationEncoding {
        self.object_reloc(is_pic).2
    }

    pub fn reloc_size(&self, is_pic: bool) -> u8 {
        self.object_reloc(is_pic).3
    }

    pub fn reloc_flags(&self, is_pic: bool) -> object::RelocationFlags {
        let reloc = self.object_reloc(is_pic);
        object::RelocationFlags::Generic {
            kind: reloc.1,
            encoding: reloc.2,
            size: reloc.3,
        }
    }
}
