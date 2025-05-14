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

