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
