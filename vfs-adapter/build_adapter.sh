cd "$(dirname "$0")"
set -ex

# Usage: build_adapter.sh [debug|release]
case "${1:-release}" in
    debug)
        PROFILE=dev
        OUT=debug
        ;;
    release)
        PROFILE=release
        OUT=release
        ;;
    *)
        echo "usage: $0 [debug|release]" >&2
        exit 1
        ;;
esac

# The adapter is a component, so its memory is handled by the component model:
# the module must not import or export memory and must not use shared memory.
#
# The `atomics` target feature is not used either: wit-bindgen does not support
# exported resources with the threads proposal (the resource type guard it
# generates asserts that the `atomics` feature is disabled) and shared memory
# requires it.
#
# The stack of the module is smaller than the default one, the adapter only
# forwards calls and does not need a megabyte of stack.
#
# RUSTFLAGS replaces the flags configured for the repository (those are meant
# for the guest modules that run inside the sandbox), see .cargo/config.toml.
RUSTFLAGS="-C target-feature=+bulk-memory,+mutable-globals -C link-arg=-zstack-size=131072" \
cargo build \
    -p webrogue-vfs-adapter \
    --profile "$PROFILE" \
    --target wasm32-unknown-unknown \
    --no-default-features \
    -Z build-std=core,alloc,panic_abort

WASM=$(realpath target/wasm32-unknown-unknown/$OUT/webrogue_vfs_adapter.wasm)
COMPONENT=$(realpath target/wasm32-unknown-unknown/$OUT/webrogue_vfs_adapter.component.wasm)

# Shrink the module with binaryen, if it is installed. Only the release build is
# optimized, the debug build keeps its debug information.
if [ "$PROFILE" = release ] && command -v wasm-opt > /dev/null 2>&1; then
    cd .. 
    cargo run --no-default-features --features=wasm-opt -- wasm-opt -Oz --enable-bulk-memory --enable-mutable-globals "$WASM" -o "$WASM.opt"
    cd vfs-adapter
    mv "$WASM.opt" "$WASM"
elif [ "$PROFILE" = release ]; then
    echo "wasm-opt not found, skipping the size optimization" >&2
fi

wasm-tools component new "$WASM" -o "$COMPONENT"

if [ "$PROFILE" = release ]; then
    mv target/wasm32-unknown-unknown/release/webrogue_vfs_adapter.component.wasm webrogue_vfs_adapter.wasm
fi