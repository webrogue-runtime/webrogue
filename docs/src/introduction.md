# Introduction

> [!WARNING]
> Webrogue documentation is work-in-progress and lots of essential content is still missing.

<!-- Welcome to Webrogue documentation! -->

## Our mission

Webrogue's ultimate goal is to let you choose any programming language, UI framework and libraries, and build your application or game without worrying that it will work on one device but not on another.
We believe that most developers don't want to fight platform-specific environment setup, face unexpected compiler and runtime errors and test changes on a number of different devices, so we abstract supported hardware and operation systems away in every aspect possible.

## Is Webrogue a right choice for you?

Webrogue aims to offer:
- Portability. Webrogue already supports Windows, Linux, macOS, Android and iOS, and support for more platforms is planned.
- Reproducibility. If it works on one device, it should work on any other.
- Simplified build process. Compile your code only once and let Webrogue manage support of every platform you target.
- Self-containing. Even if your application needs gigabytes of asset files, Webrogue compresses and embeds them. For example, any Webrogue application built for Windows turns into a single `.exe` file, while your code can read assets as if they are still real files.
- Support of theoretically any programming language. Almost any language can be compiled to WebAssembly, interpreted in WebAssembly or transpiled to another language that supports WebAssembly.

As any cross-platform solution, Webrogue has it's own constraints, although we do our best to give you as much freedom of choice as possible without sacrificing simplicity.
As for now, Webrogue expects your application to:
- Use limited API. Only general functions are supported: display, GPU access, filesystem, network, time and randomness.
It's enough for most applications, but some may want to access more exotic functionality, and Webrogue simply prevents you from calling everything your OS provides except explicitly allowed.
If you believe that there is something else that Webrogue should provide, feel free to request it in our [Issue tracker](https://github.com/webrogue-runtime/webrogue/issues).
- Be ready for sandboxing.
Your application has it's own storage and can't interference with other applications.
Note that Webrogue is not designed to safely execute untrusted code (yet seem to handle this task successfully in many cases).
- Have UI.
Applications build with Webrogue may fail to run if there is no hardware and software support to display something on screen.
A common example is a Linux machine without X11 and Wayland.
This limitation is tracked in [Webrogue Issue 200](https://github.com/webrogue-runtime/webrogue/issues/200).
- Fit different screens and support resizing.
That's just simple responsiveness.
- Run under cooperative threading.
While you can spawn as many threads as you want, not more than one thread runs at particular point in time.
It affects performance of highly-parallel CPU-intensive workloads.
[Shared-Everything Threads Proposal](https://github.com/webassembly/shared-everything-threads) promises to fix this in future.
