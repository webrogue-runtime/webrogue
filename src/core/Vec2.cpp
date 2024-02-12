#include "Vec2.hpp"

#include <math.h>

float length(Vec2<float> v) {
    return sqrtf(v.squaredLength());
}

int32_t length(Vec2<int32_t> v) {
    return ceilf(sqrtf(v.squaredLength()));
}
