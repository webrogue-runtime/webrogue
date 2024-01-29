
#if !defined(WEBROGUE_NATIVE_MODS)
#include <cstdlib>
extern "C" {
void __cxa_throw(void *ptr, void *type, void *destructor) {
    __builtin_unreachable();
}
void *__cxa_allocate_exception(size_t thrown_size) _NOEXCEPT {
    char *allocation = (char *)malloc(thrown_size);
    return allocation;
}
}
#endif
