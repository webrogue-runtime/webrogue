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

import type * as CpuRendering from "../generated/example-imports/interfaces/webrogue-gfx-cpu-rendering.js";
import type * as Vulkan from "../generated/example-imports/interfaces/webrogue-gfx-vulkan.js";
import type * as Windowing from "../generated/example-imports/interfaces/webrogue-gfx-windowing.js";
import type * as Runtime from "../generated/runtime-check/webrogue_web_runtime.js";

import {
  cpuRendering,
  vulkan,
  windowing,
  type RuntimeHost,
} from "../src/guest-runner.js";

const checks = [
  windowing satisfies typeof Windowing,
  cpuRendering satisfies typeof CpuRendering,
  vulkan satisfies typeof Vulkan,
];

// The worker exposes the wasm-bindgen exports as `webrogueWorker`. If a
// `#[wasm_bindgen(js_name = ...)]` export is removed from `web/src/host.rs`,
// the regenerated module no longer satisfies `RuntimeHost`.
const runtime = null as unknown as typeof Runtime;
const runtimeCheck: RuntimeHost = runtime;

void checks;
void runtimeCheck;
