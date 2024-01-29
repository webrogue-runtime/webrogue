
#include "base-types.h"
#include "ir.h"
#include <string>
#include <tuple>
#include <utility>
#include <vector>

using namespace std;
namespace wabt {

struct OriginsAndSymbolName {
    OriginsAndSymbolName(const WASMModule *origin, string symbol_name,
                         Address optimization_offset)
        : origin(origin), symbol_name(symbol_name),
          optimization_offset(optimization_offset) {
    }
    const WASMModule *origin;
    string symbol_name;
    Address optimization_offset;
    friend inline bool operator<(const OriginsAndSymbolName &c1,
                                 const OriginsAndSymbolName &c2) {
        if (c1.origin != c2.origin)
            return c1.origin < c2.origin;
        return c1.symbol_name < c2.symbol_name;
    }
};

struct InputSegment {
    vector<uint8_t> data;
    set<pair<const WASMModule *, Index>> original_positions;
    set<OriginsAndSymbolName> origins_and_symbol_names;
    bool is_string = false;
    Index alignment_log2;
};
// Returns the character at Pos from end of a string.
inline int charTailAt(InputSegment S, size_t Pos) {
    if (Pos >= S.data.size())
        return -1;
    return (unsigned char)S.data[S.data.size() - Pos - 1];
}

// Three-way radix quicksort. This is much faster than std::sort with strcmp
// because it does not compare characters that we already know the same.
inline void multikeySort(vector<InputSegment>::iterator Vec_begin,
                         vector<InputSegment>::iterator Vec_end, int Pos) {
tailcall:
    if ((Vec_end - Vec_begin) <= 1)
        return;

    // Partition items so that items in [0, I) are greater than the pivot,
    // [I, J) are the same as the pivot, and [J, Vec.size()) are less than
    // the pivot.
    int Pivot = charTailAt(*Vec_begin, Pos);
    size_t I = 0;
    size_t J = (Vec_end - Vec_begin);
    for (size_t K = 1; K < J;) {
        int C = charTailAt(*(Vec_begin + K), Pos);
        if (C > Pivot)
            std::swap(*(Vec_begin + I++), *(Vec_begin + K++));
        else if (C < Pivot)
            std::swap(*(Vec_begin + --J), *(Vec_begin + K));
        else
            K++;
    }

    multikeySort((Vec_begin + 0), (Vec_begin + I), Pos);
    multikeySort((Vec_begin + J), Vec_end, Pos);

    // multikeySort(Vec.slice(I, J - I), Pos + 1), but with
    // tail call optimization.
    if (Pivot != -1) {
        tie(Vec_begin, Vec_end) = make_pair(Vec_begin + I, Vec_begin + (J - I));
        ++Pos;
        goto tailcall;
    }
}
} // namespace wabt