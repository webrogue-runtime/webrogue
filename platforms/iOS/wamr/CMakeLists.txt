project(wamr C CXX ASM)
include(${WEBROGUE_ROOT_PATH}/cmake/make_vmlib.cmake)
make_vmlib(PLATFORM darwin NO_JIT)
