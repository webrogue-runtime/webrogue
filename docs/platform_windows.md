# Windows support for Webrogue

Webrogue supports compiling WRAPPs to Windows-native executable format (.exe files).

I have no Windows machine, so Windows support may be out of date. 
Your help is welcome!

All utilities needed to compile Windows-native executables are already included in Webrogue CLI utility.
It means you don't even need Windows to compile for Windows, simply run the following command:

```
webrogue compile windows <WRAPP_PATH> <OUT_PATH>
```

`--console` option allows stdout/stderr to work. 
It also makes console window appear, so this option is disabled by default.

`webrogue run` command is also available for Windows, allowing to run and debug WRAPP files.

Only x64 CPUs are currently supported.
AArch64 CPUs will be supported in future versions.

Windows versions older then Windows 10 are not supported, and not likely to be supported in nearest future.

## Implementation details

AoT artifacts for Windows are compiled using official MSVC toolchain.
CRT is linked statically.

OpenGL ES emulation is provided by [ANGLE](https://chromium.googlesource.com/angle/angle).
It is bundled as libGLESv2.dll and libEGL.dll files, and they must be in same directory as resulting .exe file.

Stripped WRAPP is bundled by appending it's content to resulting binary.
