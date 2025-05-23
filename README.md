<!-- Webrogue logo -->
<div align="center">
    <image src="https://webrogue.dev/logo.svg" width="100" height="100" alt=""/>
    <h1 align="center">
        Webrogue
    </h1>
</div>

[![Visual Studio Marketplace Last Updated](https://img.shields.io/visual-studio-marketplace/last-updated/webrogue.webrogue-vscode?label=VSCode%20extension%20release)](https://marketplace.visualstudio.com/items?itemName=webrogue.webrogue-vscode)
[![](https://img.shields.io/github/release-date/webrogue-runtime/webrogue?label=CLI%20utility%20release)](https://github.com/webrogue-runtime/webrogue/releases/latest)
[![](https://img.shields.io/github/release-date/webrogue-runtime/webrogue?label=SDK%20release)](https://github.com/webrogue-runtime/webrogue-sdk/releases/latest)
[![](https://img.shields.io/twitter/follow/WebrogueRuntime)](https://x.com/intent/follow?screen_name=WebrogueRuntime)

Webrogue is a way to port applications to different OSes with minimal effort.
Webrogue utilizes WebAssembly to allow using different programming languages instead of pinning to a specific one.
See [guides](https://webrogue.dev/guides/) to learn how to setup and use Webrogue.

The key idea is compiling and packaging applications to OS-independent format called WRAPP (WebRogue APPlication).
<!-- .wrapp -->
Same WRAPP file can be compiled to multiple OS-native formats.
Read more about compiling WRAPPs to native formats [here](docs/aot).

Webrogue already supports [Windows](docs/platform_windows), [macOS](docs/platform_xcode), [Linux](docs/platform_linux), [Android](docs/platform_android) and [iOS](docs/platform_xcode).
There are plans to support more OSes as well as running in browser.

And of cause Webrogue is open source. 
Visit [Webrogue repo](https://github.com/webrogue-runtime/webrogue) on GitHub.

## Installing Webrogue

Webrogue extension for VSCode can be found [here](https://marketplace.visualstudio.com/items?itemName=webrogue.webrogue-vscode).
Alternatively you can download [Webrogue CLI utility](https://github.com/webrogue-runtime/webrogue) and [Webrogue SDK](https://github.com/webrogue-runtime/webrogue-sdk) releases from GitHub.
