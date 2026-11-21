// Compiled by `npm run check:imports`, never bundled.
//
// `check:imports` rebuilds the wasm-bindgen exports and transpiles a real guest
// component with jco. TypeScript then compares the host implementations in
// `src/guest-runner.ts` with the interfaces that the component expects, and
// checks that the rebuilt runtime module still exposes every host function that
// `guest-runner.ts` calls. A WIT change that is not reflected in the browser
// host fails this check.
//
// WASI imports come from `@bytecodealliance/preview3-shim`, whose typings track
// a different revision of the WASI WIT files, so they are intentionally not
// compared here.

import type * as Runtime from "../generated/runtime-check/webrogue_web_runtime.js";
import type * as WebGpu from "../generated/example-imports/interfaces/wasi-webgpu-webgpu.js";
import type * as Windowing from "../generated/example-imports/interfaces/webrogue-gfx-windowing.js";
import type * as WindowingWebgpu from "../generated/example-imports/interfaces/webrogue-gfx-windowing-webgpu.js";

import * as webgpu from "../generated/wasi-gfx-shim/src/webgpu.js";
import {
  windowing,
  windowingWebgpu,
  type RuntimeHost,
} from "../src/guest-runner.js";

const checks = [
  webgpu satisfies typeof WebGpu,
  windowing satisfies typeof Windowing,
  windowingWebgpu satisfies typeof WindowingWebgpu,
];

// The worker exposes the wasm-bindgen exports as `webrogueWorker`. If a
// `#[wasm_bindgen(js_name = ...)]` export is removed from `web/src/host.rs`,
// the regenerated module no longer satisfies `RuntimeHost`.
const runtime = null as unknown as typeof Runtime;
const runtimeCheck: RuntimeHost = runtime;

void checks;
void runtimeCheck;
