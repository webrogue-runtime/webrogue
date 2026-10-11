WASI_CLANG = /sdk_dir/package/webrogue-sdk/bin/wasm32-wasip3-clang

main.wasm: main.c
	$(WASI_CLANG) --target=wasm32-wasip3 -O2 -Wl,--strip-all main.c -o main.wasm

clean:
	rm -f main.wasm

.PHONY: clean
