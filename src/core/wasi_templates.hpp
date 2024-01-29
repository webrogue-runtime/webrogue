#include "wasi_types.hpp"

namespace webrogue {
namespace core {

#define WASI_FUNCTION_IMPL(RET_TYPE, NAME, ARGS) RET_TYPE WASIObject::NAME ARGS

#ifdef NDEBUG
#define WASI_CHECK(condition)                                                  \
    if (!(condition)) {                                                        \
        return WASMRawI32::make(-1);                                           \
    }
#else
#define WASI_CHECK(condition)                                                  \
    if (!(condition)) {                                                        \
        abort();                                                               \
        return WASMRawI32::make(-1);                                           \
    }
#endif

template <typename T> struct wasi_size_helper;

template <> struct wasi_size_helper<nr_wasi_iovec_t> {
    static constexpr size_t indexOffset = 8;
};

template <typename T>
static inline uint32_t alignedOffset(uint32_t originalOffset, size_t index) {
    return originalOffset + index * wasi_size_helper<T>::indexOffset;
}

} // namespace core
} // namespace webrogue