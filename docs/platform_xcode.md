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

Both x64 (Intel) and AArch64 (Apple Silicon) are supported.
Simulators are also supported.

## Implementation details

Webrogue generates Xcode 16 project, which is used to build for both macOS and iOS.

OpenGL ES emulation is provided by [ANGLE](https://chromium.googlesource.com/angle/angle).

Stripped WRAPP is bundled as a resource file.
