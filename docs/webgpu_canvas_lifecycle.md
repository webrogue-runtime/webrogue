# WebGPU canvas lifecycle on the web

This document describes how a Webrogue window gets a DOM canvas in the browser,
why canvases are per-window, and how the canvas is re-created when a window is
closed and reopened or when the platform destroys and recreates a native window.

## Constraints

1. The guest runs in a Web Worker. It cannot touch the DOM, so it cannot render
   to the window's `<canvas>` directly.
2. A `<canvas>` element has exactly one kind of rendering context. A software
   window uses a 2D context (softbuffer), a WebGPU window uses a
   `bitmaprenderer` context. Once a context is created, it cannot be replaced.
3. A guest may open a software window, close it, and then open a WebGPU window.
   Reusing the same canvas element would leave the 2D context in place and make
   `getContext("bitmaprenderer")` return `null`.
4. A native window may be detached and re-created (platform lifecycle, surface
   loss). The guest keeps its window handle, so the guest-visible canvas id must
   stay stable across re-creation.

The design therefore gives **every window its own canvas element**, and treats
the canvas element as disposable: it is created with the window, replaced when
the native window is re-created, and removed when the window is dropped.

## Current implementation

### Id ownership

`WinitWindow` owns the canvas id (`canvas_id`), not the window attributes
callback:

- `WindowList::allocate_canvas_id()` allocates ids on the web. Ids are never
  reused, so a canvas element is never shared between two windows.
- `WinitWindow::create_native_window` stores the id in the
  `webrogue_gfx_winit::CURRENT_CANVAS_ID` atomic immediately before invoking the
  window attributes callback, because the callback does not receive the window.
- `AbstractWindow::webgpu_canvas_id()` exposes the id to the web host, which
  stores it in `HostWindow` and returns it to the worker through
  `gfxWindowWebgpuCanvasId`.
- On re-creation, `create_native_window` publishes the same id again. The
  callback replaces the canvas element under that id, so the guest's cached id
  remains valid.

### Main thread

`App::can_create_surfaces` installs a window attributes callback that:

1. reads `CURRENT_CANVAS_ID`;
2. creates a fresh `<canvas class="webrogue-canvas">`, sets
   `id="webrogue-canvas-<id>"`, sizes its backing store to the container and
   makes it fill the container (`width: 100%`, `height: 100%`, or
   `height: 100svh` when the container has no height of its own);
3. appends it to `#webrogue-canvases`;
4. inserts it into `CANVASES: BTreeMap<u32, HtmlCanvasElement>`, removing and
   detaching the previous canvas with the same id (re-creation case);
5. passes it to winit-web with `WindowAttributesWeb::with_canvas`.

`install_webgpu_port` listens on the main-thread `MessagePort`:

- `{ kind: "frame", canvas: <id>, bitmap: ImageBitmap }` →
  `ImageBitmapRenderingContext::transfer_from_image_bitmap` on `CANVASES[id]`.
  The canvas' `width`/`height` attributes are set to the bitmap size first,
  because `transferFromImageBitmap` does not resize the canvas. Unknown ids are
  ignored (stale frame after close).
- `{ kind: "drop", canvas: <id> }` → remove `CANVASES[id]` and its DOM element.

The `MessagePort` is kept alive in the `WEBGPU_PORT` thread-local. The worker
receives the other end as its `wasm_thread` payload.

### Worker

`WebgpuContext` (the `webrogue:gfx/windowing-webgpu` shim in
`web/frontend/src/guest-runner.ts`) extends the `wasi-gfx-shim` context:

- the constructor creates an `OffscreenCanvas` sized to the window;
- `configure` resizes the `OffscreenCanvas` to the current physical size before
  configuring the WebGPU context;
- `present` calls `transferToImageBitmap()` and posts the bitmap, tagged with
  `window.webgpuCanvasId()`, to the main thread.

`gfxDropWindow` removes the `HostWindow` (which closes the native window) and
posts `{ kind: "drop", canvas }` to the main thread.

## Scenarios

### Software window → close → WebGPU window

1. `makeWindow("software")` allocates canvas id `0`, creates a canvas with a 2D
   context (softbuffer) and a `WinitWindow` with `SurfaceKind::Software`.
2. `drop(window)` → `gfxDropWindow` → `HostWindow` is dropped, the native window
   closes, and the main thread removes canvas `0` from the DOM.
3. `makeWindow("webgpu")` allocates canvas id `1` and a fresh canvas element.
   `WebgpuContext` renders into an `OffscreenCanvas`; frames are displayed on
   canvas `1` through a `bitmaprenderer` context.

No context type is ever reused, so the sequence works.

### Native window re-creation (surface loss / platform lifecycle)

1. `destroy_surfaces` (or the platform equivalent) pushes `surface-lost` to the
   guest and detaches the native windows. `HostWindow` and `canvas_id` are kept.
2. The guest releases its `WGPUSurface`, creates a new one
   (`wr4c_create_webgpu_surface` → new `windowing-webgpu.Context`), and waits
   for `surface-attached`.
3. `can_create_surfaces` recreates the native window. The attributes callback
   reads the same `CURRENT_CANVAS_ID`, creates a new canvas element, and
   replaces the old one in `CANVASES` and the DOM.
4. The new `WebgpuContext` posts frames with the same id, so the main thread
   displays them on the new canvas without the guest having to learn a new id.

### Resize

The canvas fills its container (`#webrogue-canvases`), so the container decides
the size; there is no hard-coded canvas size. The web backend derives the window
size from the canvas' client size, so a container resize reaches the guest as a
window resize. `WebgpuContext::configure` then resizes the `OffscreenCanvas` to
the new physical size, the guest reconfigures its surface, and the main thread
resizes the display canvas' backing store to each frame's bitmap size. The CSS
size and the backing store are independent: the browser scales the bitmap when
the guest has not caught up with a layout change yet.

A percentage height only resolves against a container that has a height of its
own. If `#webrogue-canvases` has `height: auto` (or is absent, so the canvas is
appended to `body`), the canvas uses `height: 100svh` instead. Otherwise
`height: 100%` would become `auto`, and a canvas with `height: auto` keeps its
intrinsic aspect ratio (the ratio of its `width`/`height` attributes), freezing
the aspect ratio of the first frame. Size the container (for example with flex
`1 1 auto; min-height: 0`) to control the window size explicitly.

## Remaining work

- **Teardown visibility.** On `destroy_surfaces` the old canvas stays in the DOM
  until the native window is re-created. It should be hidden or removed when the
  native window is detached. This needs a small hook from `gfx-winit` to the web
  host (for example, a "canvas detached" message or a callback), because the
  canvas map lives in the web crate.
- **Drop ordering.** `gfxDropWindow` drops the native window before posting the
  `drop` message. A frame posted in between is ignored by the main thread, which
  is correct, but posting `drop` first would remove the canvas slightly earlier.
- **Unknown ids.** `gfxWindowWebgpuCanvasId` returns `u32::MAX` for unknown
  handles; the worker then posts frames that the main thread ignores. Throwing
  would surface guest bugs earlier.
- **Layout.** `#webrogue-canvases` is a vertical flex column. Overlapping or
  tiled windows need a real layout policy (absolute positioning, per-window
  size, z-order) if guests start opening several windows.
- **Context errors.** `get_context("bitmaprenderer")` is expected to succeed for
  a fresh canvas; failures are currently ignored. A console error would make
  misuses (for example a WebGPU context on a software canvas) easier to debug.
- **Testing.** Add a guest that opens a software window, closes it, and opens a
  WebGPU window, and exercise `simulate_destroy_surfaces` /
  `simulate_can_create_surfaces` to cover the re-creation path. The current
  webgpu example verifies the render path and the drop path (a frame limit makes
  the guest exit and drop its window; the canvas disappears).
