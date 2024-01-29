---
layout: page
title: Webrogue's modularity
---

As mentioned in explanation of Webrogue's [portability](./portable), games are distributed as __mods__. 
In context of Webrogue the word "mod" stands for "module". 

Webrogue does not differs games and modifications, so they are unified under term "mods".

Mod's code is a relocatable WebAssembly static archive. It makes it possible to merge code of all mods into one on the fly. See [linking](../in_depth/linking) for details.
