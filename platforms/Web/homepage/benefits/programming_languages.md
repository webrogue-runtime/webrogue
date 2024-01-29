---
layout: page
title: Programming-language-agnostic
---

Webrogue is designed to be programming language-agnostic.
It is about mods, not runtime itself.
Currently only C and C++ are tested, but (in theory) thous languages can be compiled to relocatable WebAssembly code and used to write mods:
 - Rust
 - Go
 - Kotlin Native
 - Swift
 - Objective-C
 - Fortran
 - Pascal

It is possible to build an interpreter and use it to run a code on an interpreted language. (In theory) It makes it possible to use:
  - JavaScript
  - Lua
  - Python
  - PHP
  - Perl
  - all jvm languages, including Java, using Oracle's HotSpot, but it will be challenging

Also it is planned to implement a [multi-stage build](../goals/milti_stage_build). 
It means that a WebAssembly code will be executed to compile some mods' resources, usually bytecode, to relocatable WebAssembly code, that will be used to link solid WebAssembly code.
(In theory) It will make it possible to use thous languages:
 - Dart
 - all dotnet languages, including C#
 - all jvm languages, including Java, using IKVM. It will be faster then using interpreter

Note that it is still possible to use thous without multi-stage build, but it will make mods less modular, so you will be not able to use to Dart mods at the same time.
