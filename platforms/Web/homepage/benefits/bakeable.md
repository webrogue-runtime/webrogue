---
layout: page
title: Webrogue's bakeability
---

As mentioned in explanation of Webrogue's [portability](./portable), games and modifications are distributed as __Webrogue mods__, which contains WebAssembly code.
But in fact it is not always so.

Game as well as modifications can be "baked" into Webrogue. 
Baking is originally used during debug process, but distribution of backed games is an option.
Webrogue offers two baking methods:

## Native method
The idea of native baking method is to compile code of mods to native code instead of WebAssembly code. It lets you to use your favorite IDE and debug utilities.
Note that if your game works natively baked it does not means that it will work when WebAssembly is used.
It is usually because natively baked code bypasses WASI.

## Wasm2c method
[Wasm2c](https://github.com/WebAssembly/wabt/blob/main/wasm2c/README.md) is an utility that, as its name says, translates WebAssembly code to C code, that is later compiled to native code. 

## Pros
- Baked games have lower startup time.
- Baked games are slightly more performant than ones that uses JIT or ATO compilation.
- Baked games generally take less disk space.

## Cons
- Usage of modifications is limited. 
Modifications that are baked in can still be used, but adding new one will require you to rebuild your game.
- You need to bake your game for each platform.
- If you are using native baking method, you may need do setup toolchains for each platform and for each compilable language used in mods.
- You generally need a source code of all mods to use native baking method.
