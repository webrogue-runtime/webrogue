#pragma once

#include <cstdint>

template <typename T> struct Vec2 {
    T x;
    T y;

    inline Vec2(T x, T y) : x(x), y(y) {
    }

    inline Vec2<T> operator*(const T multiplier) const {
        return Vec2<T>(this->x * multiplier, this->y * multiplier);
    }

    inline Vec2<T> operator/(const T divider) const {
        return Vec2<T>(this->x / divider, this->y / divider);
    }

    inline bool operator==(const Vec2<T> other) const {
        return x == other.x && y == other.y;
    }

    inline bool operator!=(Vec2<T> other) const {
        return x != other.x || y != other.y;
    }

    inline Vec2<T> operator+(const Vec2<T> other) const {
        return Vec2<T>(x + other.x, y + other.y);
    }

    inline Vec2<T> operator-(const Vec2<T> other) const {
        return Vec2<T>(x - other.x, y - other.y);
    }

    inline Vec2<T> operator*(const Vec2<T> other) const {
        return Vec2<T>(x * other.x, y * other.y);
    }

    inline T squaredLength() const {
        return this->x * this->x + this->y * this->y;
    }

    inline void operator*=(T multiplier) {
        x *= multiplier;
        y *= multiplier;
    }

    template <typename T2> explicit operator Vec2<T2>() {
        return Vec2<T2>((T2)x, (T2)y);
    }
};

float length(Vec2<float> v);

int32_t length(Vec2<int32_t> v);

typedef Vec2<int32_t> Vec2Int;
typedef Vec2<float> Vec2Float;
