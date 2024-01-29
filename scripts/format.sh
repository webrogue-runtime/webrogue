find src/ -name "*.cpp" | xargs clang-format -i
find src/ -name "*.hpp" | xargs clang-format -i
find src/ -name "*.c" | xargs clang-format -i
find src/ -name "*.h" | xargs clang-format -i

find mods/ -name "*.cpp" | xargs clang-format -i
find mods/ -name "*.hpp" | xargs clang-format -i
find mods/ -name "*.c" | xargs clang-format -i
find mods/ -name "*.h" | xargs clang-format -i