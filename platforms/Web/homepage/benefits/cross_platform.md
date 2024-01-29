---
layout: page
title: Cross-platform
---

Webrogue is a cross-platform runtime. It means that it is designed to work in several different computing platforms. Currently supported platforms are:
 - Linux
 - Windows
 - macOS
 - Android
 - iOS
 - Web.
 - DOS with limited(close to zero) support

Webrogue uses different technologies on different platforms for [rendering](../in_depth/rendering) and [code execution](../in_depth/runtimes).

## Linux
Webrogue was designed to work primary on Linux.

For code execution [Wasmtime](https://wasmtime.dev/) is used. It limits CPU architectures to x86_64 and AArch64. If you have any other CPU architecture you have to build Webrogue by yourself.

For rendering both ncurses and SDL are used. Ncurses is used by default. To use SDL, provide `--output=sdl` flag.
You can build Webrogue without SDL support too.

## Windows
Tested on Windows 11 only.

Webrogue uses [WasmEdge](https://wasmedge.org/) for code execution on Windows. The only supported CPU architecture is x86_64. Support for 32-bit windows currently is not a goal in contrast to AArch64.

Webrogue provides two executables for Windows. The first one is `webrogue_sdl.exe`, which uses SDL. The second one is named just `webrogue.exe` and uses PDCurses.

## macOS
Minimum supported macOS version is 11.0 (Big Sur).

Webrogue uses [Wasmer](https://wasmer.io/) with [JavaScriptCore](https://docs.webkit.org/Deep%20Dive/JSC/JavaScriptCore.html) backend for code execution. Both Intel and Apple Silicon chips are supported.

SDL is used for rendering. To use ncurses you need to compile Webrogue by your own.

## Android
Minimum supported Android version is 7.0 (API level 24) due to libuv limitations.

Webrogue uses [Wasmtime](https://wasmtime.dev/) on 64-bit CPUs and [WebAssembly Micro Runtime](https://bytecodealliance.github.io/wamr.dev/) with fast interpreter on 64-bit CPUs.

SDL is used for rendering.

Note for [termux](https://termux.dev/en/) users: build Webrogue by yourself just like for Linux.

## iOS
Minimum supported iOS version is 12 due to Xcode 15 limitations.

As for macOS, Webrogue uses [Wasmer](https://wasmer.io/) with [JavaScriptCore](https://docs.webkit.org/Deep%20Dive/JSC/JavaScriptCore.html) backend for code execution. It looks like JavaScriptCore does not support WebAssembly on some iOS versions, so [Wasm3](https://github.com/wasm3/wasm3) is used if Wasmer fails. Wasm3 is used as a fallback because allocation of executable memory (which is needed for more performant WebAssembly runtimes) is generally not permitted on iOS.

SDL is used for rendering.

## Web
Webrogue needs WebAssembly and SharedArrayBuffer features for dynamic loading on web, which are supported by most modern browsers. So minimum versions of browsers Webrogue supports are:
 - Chrome: 86
 - Firefox: 78
 - Safari: 14.1

Webrogue uses browser's WebAssembly feature. [Here](../in_depth/web_runtime.) is an explanation of why SharedArrayBuffer feature is also needed.
It is also possible to use interpreted runtime such as [Wasm3](https://github.com/wasm3/wasm3) to avoid browser version limitations, but it will affect performance badly.

SDL is used for rendering.

Note that [baked](./bakeable) Webrogue games support older browser versions, even without WebAssembly and SharedArrayBuffer features.

## DOS
Only [baked](./bakeable) games are supported. Wasm2c baking method is not supported due to libuv limitations. More of this, I guess Webrogue still crushes no startup, but it used to work. I do not know a reason to use DOS in 21st century, so DOS support is not a goal.

PDCurses is used for rendering.

## Other OSes
If your OS is not listed here, there is still a chance that you can build Webrogue by yourself.
