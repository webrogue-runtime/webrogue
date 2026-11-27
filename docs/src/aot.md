# Compiling WRAPPs to native formats

To publish your Webrogue application in a store and allow end users to install it with ease, WRAPP should be compiled to an OS-native format.
It is done using `webrogue compile` command.

So called AOT artifacts may be downloaded from GitHub during this process.
Webrogue comes with compiler and linker embedded into Webrogue executable, so you don't need to install other tools unless it's explicitly stated otherwise. 

## Windows

Webrogue compiles WRAPPs to standalone `.exe` files that are expected to run on Windows 10 and Windows 11 without any additional dependencies.
Just run the following command on your Windows, macOS or Linux machine:

```
webrogue compile windows <WRAPP_PATH> <OUT_PATH>
```

Webrogue supports x64 and ARM64 ISAs on WIndows, but not older 32-bit x86.
x64 is selected by default, but it can be overridden by `--arch <ARCH>` argument.

## Linux

Webrogue can compile WRAPPs into ELF files that have minimum set of dependencies using this command:

```
webrogue compile linux <WRAPP_PATH> <OUT_PATH>
```

Resulting program supports both X11 and Wayland simultaneously.

ISA can be selected using `--arch <ARCH>` argument (defaults to x86_64, also supports AArch64).
LibC can be selected using `--libc <LIBC>` (defaults to glibc, also supports musl).

### glibc

All distros with glibc 2.28+ are supported.
This includes but not limited to Debian 10, Ubuntu 18.10, Fedora 29 and later releases.
Resulting binary needs only `libc.so.6`, `libm.so.6`, `libpthread.so`, `libdl.so` and `libgcc_s.so.1` installed, and X11 or Wayland libraries to be loadable at runtime.

### musl

Webrogue links musl as a shared library, not static, so `libc.so` and `libgcc_s.so.1` must present.
Note that dependency on `libgcc_s.so.1` excludes Chimera Linux from supported distros, while most musl distros like Alpine Linux are supported.

## Android

Webrogue needs Java and Android SDK 35 to compile WRAPP to `.apk` file.
Path to Android SDK should be specified using `--sdk` option or `ANDROID_SDK_ROOT` or `ANDROID_HOME` environment variable.
Path to Java may be specified using `--java-home` option or `JAVA_HOME` environment variable.

Android apps are built using:
```
webrogue compile android <WRAPP_PATH> <BUILD_DIR>
```
...where `<BUILD_DIR>` is a path where an Android Gradle project will be generated. That path doesn't have to exist before command invocation. Resulting `.apk` file will be placed in `<BUILD_DIR>`.

> [!WARNING]
> All content in `<BUILD_DIR>` may be deleted without any warning or confirmation. Assume that `rm -rf <BUILD_DIR>` runs automatically.

List of supported ISAs can be specified using `--arch <ARCH_LIST>` option and defaults to `aarch64`.

Debug signature is used by default. To sign for release, use `--keystore-path <PATH>`, `--store-password <PASSWORD>`, `--key-password <PASSWORD>` and `--key-alias <ALIAS>` options. Key file can be generated using `webrogue android-gen-key` command or `keytool` or `openssl` utilities.

## macOS and iOS

macOS and Xcode 16 or later is required to compile WRAPP to macOS and iOS apps. Xcode is available only for macOS, so macOS machine (or CI/CD runner), is required for these steps.


At the moment of whiting this article Webrogue had no support of packaging or singing macOS/iOS apps.
These features can be polyfilled by manually archiving generated Xcode project.

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

Both x64 (Intel) and AArch64 (Apple Silicon) are supported.
Simulators are also supported.

## Implementation details

Webrogue compiles WebAssembly code Ahead-of-Time (AoT) using Wasmtime and Cranelift.
This approach combines fastest code loading, near-native execution speed and relatively low resulting binary size.
Webrogue generates object file in platform-native format, which is later linked with prebuilt Webrogue libraries as well as system libraries using [LLD](https://lld.llvm.org/) shipped with Webrogue CLI utility (or using Xcode's one), and all these things are done by just running one command.
Webrogue also automatically generates such resources as application name, ID, version, icons and splash screens form WRAPP config.
