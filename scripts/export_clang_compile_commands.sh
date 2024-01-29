cmake -DCMAKE_EXPORT_COMPILE_COMMANDS:BOOL=TRUE -S . -B build_compile_commands -DCMAKE_C_COMPILER=clang -DCMAKE_CXX_COMPILER=clang++
cmake --build build_compile_commands --target all_webrogue
cp build_compile_commands/compile_commands.json compile_commands.json
# rm -rf build_compile_commands
