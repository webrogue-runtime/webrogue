import init, * as runtime from "../generated/runtime/webrogue_web_runtime.js";
import { runGuest } from "./guest-runner.js";

const sleep = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));

(globalThis as typeof globalThis & {
    webrogueWorker?: typeof runtime & { runGuest: typeof runGuest; sleep: typeof sleep };
}).webrogueWorker = { ...runtime, runGuest, sleep };

self.onmessage = (event: MessageEvent) => {
    const [module_or_path, memory, work] = event.data;
    init({ module_or_path, memory })
        .then(() => runtime.wasm_thread_entry_point(work))
        .catch((error) => {
            console.error(error);
        });
};
