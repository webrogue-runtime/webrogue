set -ex

cargo build -v --target wasm32-wasi --release

rm -f linked.wasm
/home/someone/repos/webrogue/mods/tools/wasi-sdk-21.0/bin/wasm-ld \
    --export=wr_start \
    --export=__wasm_call_ctors \
    -z stack-size=5242880 \
    --trace --no-entry --fatal-warnings --no-gc-sections --stack-first --no-merge-data-segments \
    /home/someone/repos/webrogue/mods/core/mod.a --export=init_mod_core \
    -o linked.wasm \
    /home/someone/repos/webrogue/mods/../mods/core/stdlibs.a \
    /home/someone/repos/webrogue/mods/langExampleRust/target/wasm32-wasi/release/libhello_wasm.a --export=init_mod_langExampleRust \
    # /home/someone/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/wasm32-wasi/lib/*.rlib 
    # /home/someone/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/wasm32-wasi/lib/liballoc-68c069d1d9bd527e.rlib
    

# /home/someone/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/wasm32-wasi/lib/librustc_std_workspace_core-b21b56d0efebfda3.rlib \
#     /home/someone/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/wasm32-wasi/lib/libcore-253f7d81c4c63c8c.rlib \
    
