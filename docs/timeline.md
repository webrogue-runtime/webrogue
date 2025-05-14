# After 2 years of developing my side project has no relation to original idea

I had to host this post there because of mysterious Reddit filter rules.

TL;DR: I just tried to challenge my C++ skills by making NetHack clone, a roguelike game. Now I am releasing universal format for GUI-enabled apps that support converting to native apps for all major OSes and in theory supports almost any programming language.

## The very beginning

I found NetHack and BRogue a very addictive games despite being hardcore. I can say the same thing about C++. So I decided to develop my C++ skills by making a roguelike game. I set a few goals: support Windows, Linux, macOS, Android, iOS, Web, MSDOS, and allow to choose between Curses and SDL renderers. I quickly understood it will not be easy: different OSes has different native libraries and STLs, each kind of compiler raises it's own unique set of compile-time errors, runtime error vary for each platform and so on. Source code was filled with #if preprocessor directives and CMake configs grown in size faster than C++ codebase.

## WASM logs in

And instead of giving up I set another, maybe the most important goal: mods support. First of all I tried python, but it was really hard to cross-compile, so I moved to Lua, and later to JavaScript. It is a reasonable decision to use scripting language to make mods for games, but what if I want to write mods on Java? C#? C/C++? No one would put runtimes for each programming language. So I started to search for different approach. I was thinking about BPF or writing my own LLVM CPU target when I learned I can run WebAssembly (WASM for short) outside of a browser. I started with WASM3 interpreter, then added WAMR, WasmEdge, Wasmtime and Wasmer and browser's WebAssembly JS API support. It was not handle to pass serializable messages from one WASM module to another. So I made a hack: I was packaging mods in my own format called WRMOD (WebRogue MOD), which in turn includes resources and static libraries. These static libraries was later linked together using custom LLD-like linker right before execution. So I managed to write mods on C, C++, JavaSctipt, Rust and a few other programming languages on all supported OSes. Then I dropped MSDOS support and started to experiment with Wasm2C transpiler.

## First thoughts about Ahead-of-Time compilation

Interpreters like WASM3 was slow, while JIT-compillers like WasmEdge took ages to precompile code before it becomes runnable. Then I learned aboud Wasm2C, an utility to transpile WASM to C sources. Wasm2C become yet another WASM runtime my project supports. While Wasm2C combined fast execution and instant startup, it had a disadvantage: mods was "baked into" application, so it was not possible to just add WRMOD files, I had to rebuild the whole game to add a mod. I was frustrated: my the best supported runtime disallows modders to easily install mods.

## Rethinking and reimplementing

Now one particular game or even game engine seen to be to narrow application field. Why not to make mobile apps using the same approach? Why not to support existing UI libraries and frameworks? I asked such question while trying to figure out how to teach modders to install C/C++ toolchains and use my overcomplicated CMake configs just to give Wasm2C a try. My project had a limit: It was using Curses or Curses-like emulation, so even mobile apps looked like they are from hacker movies. I decided to drop all previous codebase and rewrite my project with slightly different set of goals: use WASM and stay cross-platform, but with fancy graphics. I decided to move to Rust to decreace number of platform-specific challenges. I quickly reimplemented WASM3, Wasmtime, Wasmer and browser's WebAssembly JS API support. I also added initial version of WebrogueGFX interface: a set of imported WebAssembly functions, enough to open a window and draw some simple graphics using OpenGL ES 2. I also got rid of mods. Now I'm using WRAPP (WebRogue APPlication) format. It's approach is simple: one file == one app.

## Ahead-of-Time compilation strikes again

I promised myself not to use Wasm2C anymore, since it's too hard for end user to install and configure C toolchain for each cross-compilation target. But when I took a closer look on Wasmer, is was enlighten. Wasmer has a "compile" CLI command. It compiles WASM code using same compiles used for JIT, emits compiled code as object file, compiles a stub C program using Zig toolchain, and links it all with precompiled static library using same Zig toolchain. So it takes WASM as input and emits runnable Window, Linux or macOS program as output. Dead simple! I took a similar approach, forked Wasmer to completely remove dependency on Zig and added support for Android and iOS. I was so inspired with new possibilities I even dropped support of all WASM runtimes but Wasmer. Then I found out that Wasmer is really bloated, consumes a lot of RAM and has no support of latest WebAssembly features, so I migrated to Wasmtime.

## Current state

Now I'm releasing Webrogue VSCode extension. This extension downloads Webrogue CLI utility and Webrogue SDK, provides some debug means and easy-to-use frontend for Webrogue CLI utility. SDK allows to easily build Webrogue Applications (WRAPP files) using C/C++. It's functionality can be extended by using additional SDKs, such as Swift SDK, to support more programming languages. SDK also includes forked versions of such libraries as GLFW and SDL2/SDL3. While some features are missing, it where enough to make such huge project as SuperTuxKart to work on top of conformant OpenGL ES 3 implementation. So it is a compile once, run everywhere solution. When WRAPP file is built, will take just one CLI invocation or a few clicks to cross-compile for Windows, Linux, Android, macOS and iOS.
When targeting Windows and Linux you don't even need to download 3td party toolchains since everything required is already embedded in 100-Megabyte Webrogue CLI utility. In case of Android you still need to install Android SDK, and Xcode is required when targeting macOS/iOS.

## Your feedback

It's the first time I'm making something so large and releasing it by my own, so your feedback is essential. There are a few guides on the website, and I would like you to give them a try and send a feedback under this post or as GitHub issue. This project is still work-in-progress, so I ultimately need someone to help me to implement missing features, but any sort of contributions including typo fixes are welcome!