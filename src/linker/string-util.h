/*
 * Copyright 2017 WebAssembly Community Group participants
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

#ifndef WABT_STRING_UTIL_H_
#define WABT_STRING_UTIL_H_

#include "StringRef.h"
#include <string>

namespace wabt {

inline std::string &operator+=(std::string &x, StringRef y) {
    x.append(y.data(), y.size());
    return x;
}

inline std::string operator+(StringRef x, StringRef y) {
    std::string s;
    s.reserve(x.size() + y.size());
    s.append(x.data(), x.size());
    s.append(y.data(), y.size());
    return s;
}

inline std::string operator+(const std::string &x, StringRef y) {
    return StringRef(x) + y;
}

inline std::string operator+(StringRef x, const std::string &y) {
    return x + StringRef(y);
}

inline std::string operator+(const char *x, StringRef y) {
    return StringRef(x) + y;
}

inline std::string operator+(StringRef x, const char *y) {
    return x + StringRef(y);
}

inline void cat_concatenate(std::string &) {
}

template <typename T, typename... Ts>
void cat_concatenate(std::string &s, const T &t, const Ts &...args) {
    s += t;
    cat_concatenate(s, args...);
}

inline size_t cat_compute_size() {
    return 0;
}

template <typename T, typename... Ts>
size_t cat_compute_size(const T &t, const Ts &...args) {
    return StringRef(t).size() + cat_compute_size(args...);
}

// Is able to concatenate any combination of string/StringRef/char*
template <typename... Ts> std::string cat(const Ts &...args) {
    std::string s;
    s.reserve(cat_compute_size(args...));
    cat_concatenate(s, args...);
    return s;
}

} // namespace wabt

#endif // WABT_STRING_UTIL_H_
