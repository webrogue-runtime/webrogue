# Integration test fixtures

Guest sources, manifests, and checked-in WebAssembly components live beside their tests under
`tests/fixtures`. Runtime tests generally pass `webrogue.json` directly to the internal runner. The
stdio test checks exact guest stdout/stderr; the persistence test runs the same guest twice against
one temporary persistent-storage root; the CPU-rendering test compares a captured PPM frame with a
checked-in golden image; and the resources tests read a bundled asset from both a `webrogue.json`
configuration and a packed WRAPP. The AOT test packs the stdio fixture because `webrogue compile`
consumes WRAPP files.

The AOT stdio test compiles and launches a native app: Linux (x86-64 or AArch64; glibc or musl),
Windows (x86-64 or AArch64), or macOS (universal app built from the bundled Xcode project). Linux
uses `aot_artifacts.zip`; macOS uses `aot_artifacts_macos.zip`. Linux runs need an X server; in
headless environments, use Xvfb. Prepare Linux artifacts for the current host with:

```sh
LIBCS=gnu COMPONENTS=template sh linux/build.sh
sh scripts/pack_aot_artifacts.sh
```

Use `LIBCS=musl` instead on a musl host.

To regenerate the guest components, build the SDK in release mode and build the examples image,
then run:

```sh
WEBROGUE_SDK_DEBUG=0 sh webrogue-sdk/build.sh
sh webrogue-sdk/examples/make_image.sh
sh webrogue-sdk/examples/build_tests.sh
```

The runtime integration tests can be run without the SDK:

```sh
cargo test --locked --no-default-features --features internal-run --test runtime
```

Run the AOT subprocess test (after preparing the artifacts above) with:

```sh
xvfb-run -a cargo test --locked --no-default-features --features compile,llvm --test aot_stdio
```

The cross-platform AOT artifact build and native Linux, Windows, and macOS test matrix can be
started from GitHub Actions via the **AOT integration tests** `workflow_dispatch` workflow. It
builds the shared artifact bundles through `build_aot_artifacts.yml`; normal CI does not build them.
