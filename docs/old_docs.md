That's just some notes copied from legacy documentation.
Most of this information is outdated.

# Android support for Webrogue

Webrogue supports compiling WRAPPs to APK files for Android.
To compile WRAPP to Android application, you need Android SDK 35.
Path to Android SDK should be specified using `--sdk` option or `ANDROID_SDK_ROOT` or `ANDROID_HOME` environment variable.
Path to Java may be specified using `--java-home` option or `JAVA_HOME` environment variable.
You can compile Android application with following command:

```
webrogue compile android <WRAPP_PATH> <BUILD_DIR> --sdk <PATH>
```

To optionally signing your APK, use `--keystore-path`, `--store-password` `--key-password` and `--key-alias` options.

Webrogue a work-in-progress app to run and debug WRAPP files on Android using JIT compilation.

Only AArch64 CPUs are currently supported.
Most Android devices are AArch64-based.
Exceptions are some old devices, ultra low-end devices, and many emulators.

## Implementation details

Webrogue generates Gradle Android project to build Android apps.
Generated project is also compatible with Android Studio.

Webrogue links compiled object and libraries shipped with `webrogue` utility, just like for Windows or Linux.
The only difference is that linking result is a shared library, which is placed to generated project.
Gradle doesn't builds any native code, only Java code, so NDK is not needed.

Android devices ships OpenGL ES drives, so no graphics emulation layer is required.

Stripped WRAPP is bundled as uncompressed Android asset.
It means it is not required to unpack it to provide non-sequential access.
WRAPP itself is a compressed file format, so lack of asset compression doesn't increases resulting APK size.

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

# Web support for Webrogue

Since Webrogue uses WebAssembly to store application's code, it is possible to run WRAPPs right in browser.
<!-- [This web runtime](../demos/dynamic/index.html) demonstrates this feature. -->
Sad thing is that Emscripten's OpenGL ES implementation seems to be non-conformant, so must apps will simply draw solid color.
<!-- Currently it allows you only to pick a WRAPP file and run it. -->
<!-- Code runs completely locally. -->

## Implementation details

Webrogue runtime for web is compiled using [Emscripten](https://emscripten.org/). 
OpenGL ES emulation is currently also provided by Emscripten and works on top of WebGL 2.
Emscripten's OpenGL ES emulation seems to be not 100% complete, so it may cause bugs while running some apps with complicated graphics.

To run multithreaded apps on web, Webrogue needs SharedArrayBuffer support, which in turn requires secure context and cross-origin isolation. 
See [SharedArrayBuffer security requirements](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/SharedArrayBuffer#security_requirements) at MDN.
Since GitHub pages doesn't offers a way to set required headers, web runtime uses [coi-serviceworker](https://github.com/gzuidhof/coi-serviceworker) to inject them.
It may cause web runtime page to instantly reload on first launch.

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

# macOS and iOS support for Webrogue

Webrogue supports compiling WRAPPs to App Store-ready macOS and iOS applications.
At the moment of whiting this article Webrogue had no support of packaging or singing macOS/iOS apps.
These features can be polyfilled by manually archiving generated Xcode project.

Xcode 16 or later is required to compile WRAPPs to macOS and iOS apps. 
Xcode is available only for macOS, so macOS machine (or CI/CD runner), is required for these steps.

Compilation is done using following commands:

```
webrogue compile xcode <WRAPP_PATH> <BUILD_DIR> macos
```

and

```
webrogue compile xcode <WRAPP_PATH> <BUILD_DIR> ios
```

Xcode project can be generated using following command:

```
webrogue compile xcode <WRAPP_PATH> <BUILD_DIR> project
```

Resulting project is placed at `<BUILD_DIR>/webrogue.xcodeproj` and can be opened in Xcode.

`webrogue run` utility is available for macOS, allowing to run and debug WRAPP files.

Webrogue also has work-in-progress app to run and debug WRAPP files on iOS, but it is not likely to work on non-jailbroken devices.

## Implementation details

Webrogue generates Xcode 16 project, which is used to build for both macOS and iOS.

OpenGL ES emulation is provided by [ANGLE](https://chromium.googlesource.com/angle/angle).

Stripped WRAPP is bundled as a resource file.
