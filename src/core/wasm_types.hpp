#pragma once

#include "byteswap.hpp"
#include <cstdint>

namespace webrogue {
namespace core {

template <typename T> class WASMNum {
private:
    T value;

public:
    static inline WASMNum<T> make(T value) {
        WASMNum<T> result;
        result.value = byteswap<T>(value);
        return result;
    }
    inline T get() const {
        return byteswap<T>(value);
    }
};

typedef WASMNum<int32_t> WASMI32;
static_assert(sizeof(WASMI32) == sizeof(int32_t), "?");

typedef WASMNum<uint32_t> WASMU32;
static_assert(sizeof(WASMU32) == sizeof(uint32_t), "?");

typedef WASMNum<int64_t> WASMI64;
static_assert(sizeof(WASMI64) == sizeof(int64_t), "?");

typedef WASMNum<uint64_t> WASMU64;
static_assert(sizeof(WASMU64) == sizeof(uint64_t), "?");

typedef WASMNum<float> WASMF32;
static_assert(sizeof(WASMF32) == sizeof(float), "?");

typedef WASMNum<double> WASMF64;
static_assert(sizeof(WASMF64) == sizeof(double), "?");

template <typename T> class WASMRawNum {
private:
    T value;

public:
    static inline WASMRawNum<T> make(T value) {
        WASMRawNum<T> result;
        result.value = value;
        return result;
    }
    inline T get() const {
        return value;
    }
};

typedef WASMRawNum<int32_t> WASMRawI32;
static_assert(sizeof(WASMRawI32) == sizeof(int32_t), "?");

typedef WASMRawNum<uint32_t> WASMRawU32;
static_assert(sizeof(WASMRawU32) == sizeof(uint32_t), "?");

typedef WASMRawNum<int64_t> WASMRawI64;
static_assert(sizeof(WASMRawI64) == sizeof(int64_t), "?");

typedef WASMRawNum<uint64_t> WASMRawU64;
static_assert(sizeof(WASMRawU64) == sizeof(uint64_t), "?");

typedef WASMRawNum<float> WASMRawF32;
static_assert(sizeof(WASMRawF32) == sizeof(float), "?");

typedef WASMRawNum<double> WASMRawF64;
static_assert(sizeof(WASMRawF64) == sizeof(double), "?");

} // namespace core
} // namespace webrogue
