# Integration test fixtures

Guest sources, manifests, and checked-in WebAssembly components live beside their tests under
`tests/fixtures`. Runtime tests generally pass `webrogue.json` directly to the internal runner. The
stdio test checks exact guest stdout/stderr; the persistence test runs the same guest twice against
one temporary persistent-storage root; the CPU-rendering test compares a captured PPM frame with a
checked-in golden image; and the resources tests read a bundled asset from both a `webrogue.json`
configuration and a packed WRAPP. The desktop AOT tests pack the stdio fixture first; Android and
iOS compile directly from the fixture's `webrogue.json`.

The AOT stdio tests compile and launch native apps on Linux (x86-64 or AArch64; glibc or musl),
Windows (x86-64 or AArch64), macOS (universal app built from the bundled Xcode project), and Android
(both AArch64 and x86-64 ABIs, launched in an x86-64 emulator) and iOS (launched in an iPhone
Simulator using the bundled Xcode project). Android output is captured from logcat; iOS output is
captured from the Simulator console. Linux uses `aot_artifacts.zip`; Apple tests use
`aot_artifacts_macos.zip`. Linux runs need an X server; in headless environments, use Xvfb. Prepare
Linux artifacts for the current host with:

```sh
LIBCS=gnu COMPONENTS=template sh linux/build.sh
sh scripts/pack_aot_artifacts.sh
```

Use `LIBCS=musl` instead on a musl host.

To prepare the Android AOT template and combined artifact archive, with the Android SDK configured,
run from the repository root:

```sh
sh android/runtime/build_native_code.sh dev
sh android/template/build_template.sh
sh scripts/pack_aot_artifacts.sh
```

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

The Android AOT subprocess test requires an x86-64 Android emulator to be running and builds a
dual-ABI APK from the same artifact bundle:

```sh
cargo test --locked --no-default-features --features compile,llvm --test aot_android
```

`webrogue compile android` defaults to AArch64. Pass `--arch aarch64,x86_64` to produce an APK
containing both architectures, or select either ABI individually.

The iOS AOT subprocess test requires Xcode and an available iPhone Simulator. The bundled Xcode
project is built directly; the test job does not need XcodeGen:

```sh
cargo test --locked --no-default-features --features compile,llvm --test aot_ios
```

The cross-platform AOT artifact build and Linux, Windows, macOS, Android, and iOS test matrix can be
started from GitHub Actions via the **AOT integration tests** `workflow_dispatch` workflow. It
builds the shared artifact bundles through `build_aot_artifacts.yml`; normal CI does not build them.
