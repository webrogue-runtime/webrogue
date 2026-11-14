import { transpile } from "@bytecodealliance/jco/component";
import {
  _setArgs,
  _setEnv,
  environment,
  exit,
  stderr,
  stdin,
  stdout,
  terminalInput,
  terminalOutput,
} from "@bytecodealliance/preview3-shim/cli";
import * as clocks from "@bytecodealliance/preview3-shim/clocks";
import * as filesystem from "@bytecodealliance/preview3-shim/filesystem";
import * as random from "@bytecodealliance/preview3-shim/random";

const symbolDispose = Symbol.dispose || Symbol.for('dispose');

export type RuntimeHost = {
  gfxMakeWindow(): number;
  gfxDropWindow(handle: number): void;
  gfxWindowPhysicalSize(handle: number): Uint32Array;
  gfxWindowLogicalSize(handle: number): Uint32Array;
  gfxWindowVulkanId(handle: number): number;
  gfxWindowNextEvent(handle: number): Promise<unknown | undefined>;
  gfxWindowSetMouseGrab(handle: number, enabled: boolean): boolean;
  gfxPresentPixels(handle: number, pixels: Uint32Array): void;
  vfsRoot(): number | undefined;
  vfsOpen(handle: number, path: string): { ok?: number; err?: string };
  vfsDrop(handle: number): void;
  vfsIsDir(handle: number): boolean;
  vfsReadAt(handle: number, offset: number, length: number): Uint8Array | undefined;
  vfsStat(handle: number): { type: string; size: number } | { err: string };
  vfsChildren(
    handle: number,
  ): { name: string; directory: boolean }[] | { err: string };
};

declare const webrogueWorker: RuntimeHost;

const TerminalOutput = terminalOutput.TerminalOutput;

const terminalStdoutOverride = {
  getTerminalStdout(): InstanceType<typeof TerminalOutput> {
    return new TerminalOutput();
  },
};

const terminalStderrOverride = {
  getTerminalStderr(): InstanceType<typeof TerminalOutput> {
    return new TerminalOutput();
  },
};

const terminalStdinOverride = {
  getTerminalStdin(): undefined {
    return undefined;
  },
};

// jco hands the resource object back to the host, so the host can keep its
// opaque handle out of the object's public shape. The generated WIT types see
// only the resource methods, not implementation details like this handle.
const windowHandles = new WeakMap<object, number>();

function windowHandle(window: object): number {
  const handle = windowHandles.get(window);
  if (handle === undefined) {
    throw new Error("window is not registered with the host");
  }
  return handle;
}

class Window {
  // Matches jco's generated resource constructor signature. Instances are only
  // created by `makeWindow` below.
  private constructor() { }

  static create(handle: number): Window {
    const window = new Window();
    windowHandles.set(window, handle);
    return window;
  }

  getPhysicalSize(): [number, number] {
    const size = webrogueWorker.gfxWindowPhysicalSize(windowHandle(this));
    return [size[0] ?? 0, size[1] ?? 0];
  }

  getLogicalSize(): [number, number] {
    const size = webrogueWorker.gfxWindowLogicalSize(windowHandle(this));
    return [size[0] ?? 0, size[1] ?? 0];
  }

  getVulkanId(): number {
    return webrogueWorker.gfxWindowVulkanId(windowHandle(this));
  }

  setMouseGrab(enabled: boolean): { tag: "ok" | "err"; val: undefined } {
    const ok = webrogueWorker.gfxWindowSetMouseGrab(
      windowHandle(this),
      enabled,
    );
    return { tag: ok ? "ok" : "err", val: undefined };
  }

  eventStream<T = unknown>(): AsyncIterable<T> {
    const handle = windowHandle(this);
    const stream: AsyncIterable<unknown> = {
      [Symbol.asyncIterator](): AsyncIterator<unknown> {
        return {
          async next(): Promise<IteratorResult<unknown>> {
            const event = await webrogueWorker.gfxWindowNextEvent(handle);
            if (event === undefined) {
              return { value: undefined, done: true };
            }
            return { value: event, done: false };
          },
        };
      },
    };
    return stream as AsyncIterable<T>;
  }

  [symbolDispose]() {
    const handle = windowHandle(this);
    webrogueWorker.gfxDropWindow(handle);
  }
}

export const windowing = {
  Window,
  makeWindow(): Window {
    return Window.create(webrogueWorker.gfxMakeWindow());
  },
};

export const cpuRendering = {
  presentPixels(window: object, pixels: Uint32Array): void {
    webrogueWorker.gfxPresentPixels(windowHandle(window), pixels);
  },
};

export const deviceInfo = {
  getOsFamily(): number {
    return 0;
  },
};

export const vulkan = {
  checkPresence(): boolean {
    return false;
  },
  Renderer: class Renderer {
    constructor(_name: Uint8Array) { }

    registerBlob(_resId: number, _buf: Uint8Array): void { }
    createShmem(_buf: Uint8Array): number {
      throw new Error("Vulkan is not available in the web runtime");
    }
    createDeviceMemoryBlob(_blobId: bigint, _size: bigint): number {
      throw new Error("Vulkan is not available in the web runtime");
    }
    resourceUnref(_resId: number): void { }
    createSync(_value: bigint): number {
      throw new Error("Vulkan is not available in the web runtime");
    }
    syncUnref(_syncId: number): void { }
    syncRead(_syncId: number): bigint {
      throw new Error("Vulkan is not available in the web runtime");
    }
    syncWrite(_syncId: number, _value: bigint): void { }
    submitCmd(
      _headers: Uint8Array,
      _cmds: Uint8Array,
      _syncs: Uint8Array,
    ): void {
      throw new Error("Vulkan is not available in the web runtime");
    }
    syncWait(
      _flags: number,
      _timeout: number,
      _syncs: Uint32Array,
    ): number {
      throw new Error("Vulkan is not available in the web runtime");
    }
    getMaxTimelineCount(): number {
      return 0;
    }
    getCapset(_id: number, _version: number): Uint8Array {
      return new Uint8Array();
    }
    contextInit(_capsetId: number): void {
      throw new Error("Vulkan is not available in the web runtime");
    }
  },
};

const vfsOk = { tag: "ok", val: undefined };

const BYTE_CHUNK_SIZE = 64 * 1024;

/**
 * Reads a descriptor in bounded chunks, on demand.
 *
 * A `stream<u8>` lowered from a byte-at-a-time async iterator makes the guest
 * wait for the component scheduler on every byte. Font loading alone does
 * thousands of reads, so that turns into minutes of wall-clock time. A
 * synchronous iterator lets jco drain a read without suspending the guest, and
 * asking the VFS for one chunk at a time keeps large files out of memory.
 */
function byteStream(handle: number, start: number): Iterable<Uint8Array> {
  const first = Math.min(start, Number.MAX_SAFE_INTEGER);
  return {
    [Symbol.iterator](): Iterator<Uint8Array> {
      let offset = first;
      let done = false;
      return {
        next(): IteratorResult<Uint8Array> {
          if (done) {
            return { value: undefined, done: true };
          }
          const chunk = webrogueWorker.vfsReadAt(handle, offset, BYTE_CHUNK_SIZE);
          if (chunk === undefined || chunk.length === 0) {
            done = true;
            return { value: undefined, done: true };
          }
          offset += chunk.length;
          return { value: chunk, done: false };
        },
      };
    },
  };
}

function entriesStream(entries: unknown[]): Iterable<unknown> {
  return {
    [Symbol.iterator](): Iterator<unknown> {
      let index = 0;
      return {
        next(): IteratorResult<unknown> {
          if (index >= entries.length) {
            return { value: undefined, done: true };
          }
          const value = entries[index];
          index += 1;
          return { value, done: false };
        },
      };
    },
  };
}

class VfsDescriptor {
  constructor(readonly handle: number) { }

  open(path: string): VfsDescriptor {
    const result = webrogueWorker.vfsOpen(this.handle, path);
    if (result.err !== undefined || result.ok === undefined) {
      throw { tag: result.err ?? "other" };
    }
    return new VfsDescriptor(result.ok);
  }

  readAt(
    offset: bigint,
  ): [Iterable<Uint8Array>, PromiseLike<unknown>] {
    if (this.isDir()) {
      return [
        byteStream(this.handle, 0),
        Promise.resolve({ tag: "err", val: { tag: "is-directory" } }),
      ];
    }
    return [
      byteStream(this.handle, Number(offset)),
      Promise.resolve(vfsOk),
    ];
  }

  writeAt(): PromiseLike<unknown> {
    return Promise.resolve({ tag: "err", val: { tag: "read-only" } });
  }

  stat(): unknown {
    const stat = webrogueWorker.vfsStat(this.handle) as {
      type: string;
      size: number;
    };
    if ("err" in stat) {
      throw { tag: (stat as unknown as { err: string }).err };
    }
    return {
      type: { tag: stat.type },
      linkCount: 1n,
      size: BigInt(stat.size),
    };
  }

  isDir(): boolean {
    return webrogueWorker.vfsIsDir(this.handle);
  }

  dirChildren(): [Iterable<unknown>, PromiseLike<unknown>] {
    const children = webrogueWorker.vfsChildren(this.handle);
    if (!Array.isArray(children)) {
      return [
        entriesStream([]),
        Promise.resolve({ tag: "err", val: { tag: "not-directory" } }),
      ];
    }
    const entries = children.map((child) => ({
      name: child.name,
      type: { tag: child.directory ? "directory" : "regular-file" },
    }));
    return [entriesStream(entries), Promise.resolve(vfsOk)];
  }

  [symbolDispose]() {
    const handle = windowHandle(this);
    webrogueWorker.vfsDrop(this.handle);
  }
}

export const vfsHost = {
  Descriptor: VfsDescriptor,
  rootDescriptor(): VfsDescriptor {
    const handle = webrogueWorker.vfsRoot();
    if (handle === undefined) {
      throw { tag: "other" };
    }
    return new VfsDescriptor(handle);
  },
};

export const wasiImports = {
  "wasi:cli/environment": environment,
  "wasi:cli/exit": exit,
  "wasi:cli/stderr": stderr,
  "wasi:cli/stdin": stdin,
  "wasi:cli/stdout": stdout,
  "wasi:cli/terminal-input": terminalInput,
  "wasi:cli/terminal-output": terminalOutput,
  "wasi:cli/terminal-stdin": terminalStdinOverride,
  "wasi:cli/terminal-stdout": terminalStdoutOverride,
  "wasi:cli/terminal-stderr": terminalStderrOverride,
  "wasi:clocks/monotonic-clock": clocks.monotonicClock,
  "wasi:clocks/system-clock": clocks.systemClock,
  "wasi:clocks/types": clocks.types,
  "wasi:filesystem/preopens": filesystem.preopens,
  "wasi:filesystem/types": filesystem.types,
  "wasi:random/random": random.random,
  "wasi:random/insecure": random.insecure,
  "wasi:random/insecure-seed": random.insecureSeed,
};

type ImportObject = Record<string, unknown>;

function buildImports(): ImportObject {
  const imports: ImportObject = {
    ...wasiImports,
    "webrogue:gfx/windowing": windowing,
    "webrogue:gfx/cpu-rendering": cpuRendering,
    "webrogue:gfx/device-info": deviceInfo,
    "webrogue:gfx/vulkan": vulkan,
    "webrogue:vfs/vfs": vfsHost,
  };

  for (const [name, value] of Object.entries(wasiImports)) {
    for (const version of ["0.3.0", "0.3.0-rc-2026-03-15"]) {
      imports[`${name}@${version}`] ??= value;
    }
  }

  return imports;
}

type GeneratedModule = {
  instantiate(
    getCoreModule: (path: string) => WebAssembly.Module,
    imports: ImportObject,
  ): unknown | Promise<unknown>;
};

export async function runGuest(componentBytes: Uint8Array): Promise<void> {
  _setArgs([]);
  _setEnv({});

  const generated = await transpile(componentBytes, {
    name: "main",
    map: [],
    instantiation: { tag: "async" },
    asyncMode: { tag: "jspi", val: { imports: [], exports: [] } },
    validLiftingOptimization: false,
    tracing: false,
    noComponentErrorWrapping: true,
    noNodejsCompat: true,
    noTypescript: true,
    tlaCompat: false,
    base64Cutoff: 5000,
    noNamespacedExports: false,
    multiMemory: false,
    bindgenEnableWasmExnref: true,
    strict: false,
    perfStringsSkipCopyUtf8Validation: false,
    perfStringsSkipCopyUtf16Validation: false,
    flagsAsBigint: false,
    variantsInlineCases: false,
    useNamespaceObjects: false,
    enumValuesScreamingSnakeCase: false,
    asmjs: false,
    emitConditionalSuspendingWasm: false,
  });

  const jsFile = generated.files.find(([name]) => name.endsWith(".js"));
  if (jsFile === undefined) {
    throw new Error("jco did not produce a JavaScript module");
  }

  // jco also clears thread context slot 0, which wasi-libc uses as the stack
  // pointer, in its task hook while lowering a guest call. That happens, for
  // example, when a key event carries a `list<u8>` and jco calls the guest's
  // `realloc`: the guest observes a zero stack pointer and traps on its next
  // allocation. Keep the last non-zero stack pointer per thread and use it
  // whenever the slot reads zero.
  let generatedSource = new TextDecoder().decode(jsFile[1] as BufferSource);
  const stackPointerGet = "return thread.storage[slot];";
  const stackPointerSet = "thread.storage[slot] = value;";
  if (
    !generatedSource.includes(stackPointerGet) ||
    !generatedSource.includes(stackPointerSet)
  ) {
    throw new Error(
      "jco output no longer contains the expected thread context accessors; update the stack-pointer workaround",
    );
  }
  generatedSource = generatedSource
    .replace(
      stackPointerGet,
      "if (slot === 0 && thread.storage[slot] === 0 && thread.__lastSp !== undefined) {\n" +
      "  return thread.__lastSp;\n" +
      "}\n" +
      "return thread.storage[slot];",
    )
    .replace(
      stackPointerSet,
      "if (slot === 0 && value !== 0) thread.__lastSp = value;\n" +
      "thread.storage[slot] = value;",
    );

  const source =
    "const __zeroDelayQueue = [];\n" +
    "const __zeroDelayChannel = new MessageChannel();\n" +
    "__zeroDelayChannel.port1.onmessage = () => {\n" +
    "  const fn = __zeroDelayQueue.shift();\n" +
    "  if (fn !== undefined) fn();\n" +
    "};\n" +
    "const setTimeout = (fn, delay, ...args) => {\n" +
    "  if (delay === 0) {\n" +
    "    __zeroDelayQueue.push(() => fn(...args));\n" +
    "    __zeroDelayChannel.port2.postMessage(0);\n" +
    "    return 0;\n" +
    "  }\n" +
    "  return globalThis.setTimeout(fn, delay, ...args);\n" +
    "};\n" + // boosts zero-delay setTimeout
    generatedSource;

  const coreModules = new Map<string, WebAssembly.Module>();
  for (const [name, bytes] of generated.files) {
    if (name.endsWith(".wasm")) {
      coreModules.set(
        name.replace(/^\.\//, ""),
        new WebAssembly.Module(bytes as BufferSource),
      );
    }
  }

  const moduleUrl = URL.createObjectURL(
    new Blob([source], { type: "text/javascript" }),
  );

  try {
    const module = (await import(
      /* @vite-ignore */ moduleUrl
    )) as GeneratedModule;

    const root = (await module.instantiate(
      (path) => {
        const normalized = path.replace(/^\.\//, "");
        const coreModule = coreModules.get(normalized);
        if (coreModule === undefined) {
          throw new Error(`missing core module ${path}`);
        }
        return coreModule;
      },
      buildImports(),
    )) as Record<string, { run(): Promise<void> } | undefined>;

    const entry = root["wasi:cli/run@0.3.0"] ?? root.run;
    if (entry === undefined) {
      throw new Error("transpiled component does not export wasi:cli/run");
    }
    await entry.run();
  } finally {
    URL.revokeObjectURL(moduleUrl);
  }
}
