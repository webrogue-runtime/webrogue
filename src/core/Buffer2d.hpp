#pragma once

#include "Vec2.hpp"
#include <vector>

template <typename T> struct Buffer2d {
private:
    std::vector<T> buffer;
    int height = 0;
    int width = 0;

public:
    inline void resize(Vec2Int size) {
        width = size.x;
        height = size.y;
        buffer.resize(height * width);
    }

    inline void fill(T value) {
        buffer.assign(buffer.size(), value);
    }

    inline Vec2Int size() const {
        return Vec2Int(width, height);
    }

    inline T *data() {
        return buffer.data();
    }

    inline const T at(int x, int y) const {
        return buffer[x + width * y];
    }

    inline T &at(int x, int y) {
        return buffer[x + width * y];
    }

    inline const T at(Vec2Int v) const {
        return at(v.x, v.y);
    }

    inline T &at(Vec2Int v) {
        return at(v.x, v.y);
    }
};
