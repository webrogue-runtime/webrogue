#pragma once

#include <cstdint>
#if defined(_MSC_VER)
#define WEBROGUE_LITTLE_ENDIAN
#elif defined(__BYTE_ORDER__) && __BYTE_ORDER__ == __ORDER_LITTLE_ENDIAN__
#define WEBROGUE_LITTLE_ENDIAN
#elif defined(__BYTE_ORDER__) && __BYTE_ORDER__ == __ORDER_BIG_ENDIAN__
#define WEBROGUE_BIG_ENDIAN
#else
#error "Byte order not detected"
#endif

#if defined(WEBROGUE_BIG_ENDIAN)
#define WEBROGUE_BSWAP_u8(X)                                                   \
    {}
#define WEBROGUE_BSWAP_u16(X)                                                  \
    { (X) = WEBROGUE_bswap16((X)); }
#define WEBROGUE_BSWAP_u32(X)                                                  \
    { (X) = WEBROGUE_bswap32((X)); }
#define WEBROGUE_BSWAP_u64(X)                                                  \
    { (X) = WEBROGUE_bswap64((X)); }
#define WEBROGUE_BSWAP_i8(X)                                                   \
    {}
#define WEBROGUE_BSWAP_i16(X) WEBROGUE_BSWAP_u16(X)
#define WEBROGUE_BSWAP_i32(X) WEBROGUE_BSWAP_u32(X)
#define WEBROGUE_BSWAP_i64(X) WEBROGUE_BSWAP_u64(X)
#define WEBROGUE_BSWAP_f32(X)                                                  \
    {                                                                          \
        union {                                                                \
            f32 f;                                                             \
            u32 i;                                                             \
        } u;                                                                   \
        u.f = (X);                                                             \
        WEBROGUE_BSWAP_u32(u.i);                                               \
        (X) = u.f;                                                             \
    }
#define WEBROGUE_BSWAP_f64(X)                                                  \
    {                                                                          \
        union {                                                                \
            f64 f;                                                             \
            u64 i;                                                             \
        } u;                                                                   \
        u.f = (X);                                                             \
        WEBROGUE_BSWAP_u64(u.i);                                               \
        (X) = u.f;                                                             \
    }
#elif defined(WEBROGUE_LITTLE_ENDIAN)
#define WEBROGUE_BSWAP_u8(X)                                                   \
    {}
#define WEBROGUE_BSWAP_u16(x)                                                  \
    {}
#define WEBROGUE_BSWAP_u32(x)                                                  \
    {}
#define WEBROGUE_BSWAP_u64(x)                                                  \
    {}
#define WEBROGUE_BSWAP_i8(X)                                                   \
    {}
#define WEBROGUE_BSWAP_i16(X)                                                  \
    {}
#define WEBROGUE_BSWAP_i32(X)                                                  \
    {}
#define WEBROGUE_BSWAP_i64(X)                                                  \
    {}
#define WEBROGUE_BSWAP_f32(X)                                                  \
    {}
#define WEBROGUE_BSWAP_f64(X)                                                  \
    {}
#else
#error "Byte order not detected"
#endif

template <typename T> inline T byteswap(T value);

template <> inline int8_t byteswap<int8_t>(int8_t value) {
    WEBROGUE_BSWAP_i8(value);
    return value;
}

template <> inline int16_t byteswap<int16_t>(int16_t value) {
    WEBROGUE_BSWAP_i16(value);
    return value;
}

template <> inline int32_t byteswap<int32_t>(int32_t value) {
    WEBROGUE_BSWAP_i32(value);
    return value;
}

template <> inline int64_t byteswap<int64_t>(int64_t value) {
    WEBROGUE_BSWAP_i64(value);
    return value;
}

template <> inline uint8_t byteswap<uint8_t>(uint8_t value) {
    WEBROGUE_BSWAP_u8(value);
    return value;
}

template <> inline uint16_t byteswap<uint16_t>(uint16_t value) {
    WEBROGUE_BSWAP_u16(value);
    return value;
}

template <> inline uint32_t byteswap<uint32_t>(uint32_t value) {
    WEBROGUE_BSWAP_u32(value);
    return value;
}

template <> inline uint64_t byteswap<uint64_t>(uint64_t value) {
    WEBROGUE_BSWAP_u64(value);
    return value;
}

template <> inline float byteswap<float>(float value) {
    static_assert(sizeof(float) == 4, "float is not f32");
    WEBROGUE_BSWAP_f32(value);
    return value;
}

template <> inline double byteswap<double>(double value) {
    static_assert(sizeof(double) == 8, "double is not f64");
    WEBROGUE_BSWAP_f64(value);
    return value;
}
