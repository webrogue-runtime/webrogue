---
layout: page
title: Linking
---

Mods contains static archives with relocatable WebAssembly code. 
It makes it possible to merge code of all mods into one.
It means that if you want to add functionality to existing game, you can just add mod without rebuilding the whole game.

Webrogue links mods statically, so mods have co preserve [relocations](https://en.wikipedia.org/wiki/Relocation_(computing)) in WebAssembly code.
Webrogue does not supports dynamic linking because this method is not mature enough, but may start supporting it in future.

WebAssembly code is usually linked using [LLD](https://lld.llvm.org/). 
But it appears that LLD is to hard to compile, and it is making Webrogue's binary too big.
So Webrogue uses its own static linker.
Webrogue's linker tries to mimic LLD, but can link WebAssembly code only and lucks some features, such as LTO.
The linker is build on top of modified [WABT](https://github.com/WebAssembly/wabt) codebase.
It makes Webrogue's linker much more portable than LLD.
