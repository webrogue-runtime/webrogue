cd $(dirname $0)
set -ex


sh build_linux_aot_artifacts.sh

cargo run \
    --manifest-path=../crates/aot-compiler/Cargo.toml \
    --target-dir=./target \
    --release \
    ../examples/gears/gears.webc \
    wr_aot.o \
    x86_64-linux-gnu

ld.lld \
    "-z" \
    "relro" \
    "--hash-style=gnu" \
    "--build-id" \
    "--eh-frame-hdr" \
    "-m" "elf_x86_64" \
    "-pie" \
    "-dynamic-linker" "/lib64/ld-linux-x86-64.so.2" \
    "-o" "a.out" \
    "../aot_artifacts/x86_64-linux-gnu/libwebrogue_aot_lib.a" \
    "wr_aot.o" \
    "../aot_artifacts/x86_64-linux-gnu/libm.so.6" \
    "--as-needed" "../aot_artifacts/x86_64-linux-gnu/libc.so.6" \
    "../aot_artifacts/x86_64-linux-gnu/libgcc_s.so.1" \
    "--no-as-needed" "../aot_artifacts/x86_64-linux-gnu/crtendS.o" 


./a.out || true
# rm -f wr_aot.o
# rm -f a.out
