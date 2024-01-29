/*
 * Copyright 2016 WebAssembly Community Group participants
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

#ifndef WABT_CONFIG_H_
#define WABT_CONFIG_H_

#include <stdint.h>

#define WABT_VERSION_STRING "1.0.33"

/* TODO(binji): nice way to define these with WABT_ prefix? */

/* Whether the target architecture is big endian */

#if __BYTE_ORDER == __LITTLE_ENDIAN
#define WABT_BIG_ENDIAN 0
#elif __BYTE_ORDER == __BIG_ENDIAN
#define WABT_BIG_ENDIAN 1
#else
#error Unknown endianness
#endif

#ifdef _MSC_VER
#define COMPILER_IS_MSVC 1
#define COMPILER_IS_CLANG 0
#define COMPILER_IS_GNU 0
#else
#define COMPILER_IS_MSVC 0
#define COMPILER_IS_CLANG 1
#define COMPILER_IS_GNU 0
#endif

#define WITH_EXCEPTIONS 0

#if COMPILER_IS_MSVC
#include <malloc.h>
#define alloca _alloca
#elif defined(__MINGW32__)
#include <malloc.h>
#else
#include <alloca.h>
#endif

#if COMPILER_IS_CLANG || COMPILER_IS_GNU

#define WABT_UNLIKELY(x) __builtin_expect(!!(x), 0)
#define WABT_LIKELY(x) __builtin_expect(!!(x), 1)

#define WABT_VECTORCALL

#if __MINGW32__
// mingw defaults to printf format specifier being ms_printf (which doesn't
// understand 'llu', etc.) We always want gnu_printf, and force mingw to always
// use mingw_printf, mingw_vprintf, etc.
#define WABT_PRINTF_FORMAT(format_arg, first_arg)                              \
    __attribute__((format(gnu_printf, (format_arg), (first_arg))))
#else
#define WABT_PRINTF_FORMAT(format_arg, first_arg)                              \
    __attribute__((format(printf, (format_arg), (first_arg))))
#endif

#ifdef __cplusplus
#define WABT_STATIC_ASSERT(x) static_assert((x), #x)
#else
#define WABT_STATIC_ASSERT(x) _Static_assert((x), #x)
#endif

#elif COMPILER_IS_MSVC

#include <intrin.h>
#include <string.h>

#define WABT_STATIC_ASSERT(x) _STATIC_ASSERT(x)
#define WABT_UNLIKELY(x) (x)
#define WABT_LIKELY(x) (x)
#define WABT_PRINTF_FORMAT(format_arg, first_arg)

#define WABT_VECTORCALL __vectorcall

#else

#error unknown compiler

#endif

#define WABT_UNREACHABLE abort()

#ifdef __cplusplus

namespace wabt {

#if COMPILER_IS_CLANG || COMPILER_IS_GNU

inline int Clz(unsigned x) {
    return x ? __builtin_clz(x) : sizeof(x) * 8;
}
inline int Clz(unsigned long x) {
    return x ? __builtin_clzl(x) : sizeof(x) * 8;
}
inline int Clz(unsigned long long x) {
    return x ? __builtin_clzll(x) : sizeof(x) * 8;
}

inline int Ctz(unsigned x) {
    return x ? __builtin_ctz(x) : sizeof(x) * 8;
}
inline int Ctz(unsigned long x) {
    return x ? __builtin_ctzl(x) : sizeof(x) * 8;
}
inline int Ctz(unsigned long long x) {
    return x ? __builtin_ctzll(x) : sizeof(x) * 8;
}

inline int Popcount(uint8_t x) {
    return __builtin_popcount(x);
}
inline int Popcount(unsigned x) {
    return __builtin_popcount(x);
}
inline int Popcount(unsigned long x) {
    return __builtin_popcountl(x);
}
inline int Popcount(unsigned long long x) {
    return __builtin_popcountll(x);
}

#elif COMPILER_IS_MSVC

#if _M_IX86
inline unsigned long LowDword(unsigned __int64 value) {
    return (unsigned long)value;
}

inline unsigned long HighDword(unsigned __int64 value) {
    unsigned long high;
    memcpy(&high, (unsigned char *)&value + sizeof(high), sizeof(high));
    return high;
}
#endif

inline int Clz(unsigned long mask) {
    if (mask == 0)
        return 32;

    unsigned long index;
    _BitScanReverse(&index, mask);
    return sizeof(unsigned long) * 8 - (index + 1);
}

inline int Clz(unsigned int mask) {
    return Clz((unsigned long)mask);
}

inline int Clz(unsigned __int64 mask) {
#if _M_X64 || _M_ARM64
    if (mask == 0)
        return 64;

    unsigned long index;
    _BitScanReverse64(&index, mask);
    return sizeof(unsigned __int64) * 8 - (index + 1);
#elif _M_IX86
    int result = Clz(HighDword(mask));
    if (result == 32)
        result += Clz(LowDword(mask));

    return result;
#else
#error unexpected architecture
#endif
}

inline int Ctz(unsigned long mask) {
    if (mask == 0)
        return 32;

    unsigned long index;
    _BitScanForward(&index, mask);
    return index;
}

inline int Ctz(unsigned int mask) {
    return Ctz((unsigned long)mask);
}

inline int Ctz(unsigned __int64 mask) {
#if _M_X64 || _M_ARM64
    if (mask == 0)
        return 64;

    unsigned long index;
    _BitScanForward64(&index, mask);
    return index;
#elif _M_IX86
    int result = Ctz(LowDword(mask));
    if (result == 32)
        result += Ctz(HighDword(mask));

    return result;
#else
#error unexpected architecture
#endif
}

#if _M_ARM64
// https://stackoverflow.com/a/70012905
template <typename T> int BrianKernighanPopcount(T value) {
    int count;
    for (count = 0; value; count++) {
        value &= value - 1;
    }
    return count;
}
#endif

inline int Popcount(unsigned long value) {
#if _M_X64 || _M_IX86
    return __popcnt(value);
#elif _M_ARM64
    return BrianKernighanPopcount(value);
#else
#error unexpected architecture
#endif
}

inline int Popcount(uint8_t value) {
    return Popcount((unsigned long)value);
}

inline int Popcount(unsigned int value) {
    return Popcount((unsigned long)value);
}

inline int Popcount(unsigned __int64 value) {
#if _M_X64
    return __popcnt64(value);
#elif _M_IX86
    return Popcount(HighDword(value)) + Popcount(LowDword(value));
#elif _M_ARM64
    return BrianKernighanPopcount(value);
#else
#error unexpected architecture
#endif
}

#else

#error unknown compiler

#endif

} // namespace wabt

#if COMPILER_IS_MSVC
/* can't just use vsnprintf because it doesn't always null terminate */
int wabt_vsnprintf(char *str, size_t size, const char *format, va_list ap);
#else
#define wabt_vsnprintf vsnprintf
#endif

#if COMPILER_IS_MSVC
/* define ssize_t identically to how LLVM does, to avoid conflicts if including
 * both */
#if defined(_WIN64)
typedef signed __int64 ssize_t;
#else
typedef signed int ssize_t;
#endif /* _WIN64 */
#else
// typedef long ssize_t;
#endif

double wabt_convert_uint64_to_double(uint64_t x);
float wabt_convert_uint64_to_float(uint64_t x);
double wabt_convert_int64_to_double(int64_t x);
float wabt_convert_int64_to_float(int64_t x);

#endif // __cplusplus

#endif /* WABT_CONFIG_H_ */
