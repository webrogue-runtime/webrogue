WASI_CLANG = /sdk_dir/package/webrogue-sdk/bin/wasm32-wasip3-clang
PKG_CONFIG ?= pkg-config
PKG_CFLAGS = $(shell $(PKG_CONFIG) --cflags libwr4c)
PKG_LIBS = $(shell $(PKG_CONFIG) --libs libwr4c)

main.wasm: main.c
	$(WASI_CLANG) --target=wasm32-wasip3 -O2 -Wl,--strip-all $(PKG_CFLAGS) main.c $(PKG_LIBS) -o main.wasm

clean:
	rm -f main.wasm

.PHONY: clean
