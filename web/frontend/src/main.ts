import init, { start } from "../generated/runtime/webrogue_web_runtime.js";

const statusElement =
  document.querySelector<HTMLParagraphElement>("#status") ??
  (() => {
    throw new Error("#status element is missing");
  })();
const fileInput =
  document.querySelector<HTMLInputElement>("#wrapp-file") ??
  (() => {
    throw new Error("#wrapp-file element is missing");
  })();
const restartButton =
  document.querySelector<HTMLButtonElement>("#restart") ??
  (() => {
    throw new Error("#restart element is missing");
  })();

const params = new URLSearchParams(location.search);
const defaultWrappUrl = params.get("wrapp");

const runtimeShimUrl = new URL(
  "../generated/runtime/webrogue_web_runtime.js",
  import.meta.url,
).href;
const guestRunnerUrl = new URL("./guest-runner.js", import.meta.url).href;

let runtimeReady: Promise<typeof import("../generated/runtime/webrogue_web_runtime.js")> | null =
  null;

function loadRuntime() {
  runtimeReady ??= init().then(() => import("../generated/runtime/webrogue_web_runtime.js"));
  return runtimeReady;
}

/**
 * The worker script reuses the wasm-bindgen shim so the Rust code running in
 * the worker shares the main thread's module and memory.
 */
function createWorkerUrl(): string {
  const source = `
import init, * as runtime from ${JSON.stringify(runtimeShimUrl)};
import { runGuest } from ${JSON.stringify(guestRunnerUrl)};

const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms));
globalThis.webrogueWorker = { ...runtime, runGuest, sleep };

self.onmessage = (event) => {
  const [module_or_path, memory, work] = event.data;
  init({ module_or_path, memory })
    .then(() => runtime.wasm_thread_entry_point(work))
    .catch((error) => {
      console.error(error);
    });
};
`;

  return URL.createObjectURL(
    new Blob([source], { type: "text/javascript" }),
  );
}

function setStatus(message: string, isError = false): void {
  statusElement.textContent = message;
  statusElement.classList.toggle("error", isError);
}

async function startWithUrl(wrappUrl: string): Promise<void> {
  setStatus("Loading runtime…");
  await loadRuntime();

    setStatus("Running.");
  const workerUrl = createWorkerUrl();
  try {
    await start(wrappUrl, workerUrl);
  } catch (error) {
    URL.revokeObjectURL(workerUrl);
    throw error;
  }
}

function lockUi(): void {
  fileInput.disabled = true;
  fileInput.closest("label")?.classList.add("disabled");
}

fileInput.addEventListener("change", async () => {
  const file = fileInput.files?.[0];
  if (file === undefined) {
    return;
  }
  lockUi();
  const fileUrl = URL.createObjectURL(file);
  try {
    await startWithUrl(fileUrl);
    setStatus("Running.");
  } catch (error) {
    console.error(error);
    setStatus(
      `Failed to start: ${error instanceof Error ? error.message : String(error)}`,
      true,
    );
  } finally {
    URL.revokeObjectURL(fileUrl);
  }
});

restartButton.addEventListener("click", () => {
  location.reload();
});

// A page load can only start the guest module once; use Reload for a
// different WRAPP.
if (defaultWrappUrl !== null) {
  startWithUrl(defaultWrappUrl)
    .then(() => lockUi())
    .catch((error: unknown) => {
      console.error(error);
      setStatus(
        `${error instanceof Error ? error.message : String(error)} — choose a local .wrapp file to start.`,
        true,
      );
    });
} else {
  setStatus("Choose a WRAPP to start.");
}
