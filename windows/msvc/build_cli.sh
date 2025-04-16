cd $(dirname $0)
set -ex

OUT_DIR="../../aot_artifacts/x86_64-windows-msvc"
rm -rf "$OUT_DIR"

XWIN_PATH="$(pwd)/xwin"
test -d "$XWIN_PATH" || xwin --accept-license splat --output "$XWIN_PATH"

cargo xwin build --target-dir=./target --target=x86_64-pc-windows-msvc --features=llvm --profile cli
