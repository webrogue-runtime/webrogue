# Linux support for Webrogue

Webrogue supports compiling WRAPPs to ELF files for Linux.
Everything needed to compile Linux-native executables is already included in Webrogue CLI utility.
It means you need neither Linux nor Docker to compile for Linux.
Just run the following command:

```
webrogue compile linux <WRAPP_PATH> <OUT_PATH>
```

`webrogue run` command is also available for Linux, allowing to run and debug WRAPP files.

Currently only x64 CPUs and glibc 2.28+ distros (like Ubuntu-18 or later) are supported.
More CPU and distro support will arrive in future versions.

## Implementation details

AoT artifacts for Linux are compiled with GCC-14 in "manylinux" Docker image.

Since most Linux distros include Mesa's OpenGL ES drives, no graphics emulation layer is required.

Stripped WRAPP is bundled by appending it's content to resulting binary (à la AppImage).
