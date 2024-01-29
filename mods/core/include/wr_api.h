#include "macros.h"
#include <stdint.h>
#define WR_API_FUNCTION(RET_TYPE, NAME, ARGS) WR_IMPORTED(RET_TYPE, NAME) ARGS;
#ifdef __cplusplus
extern "C" {
#define EXTERN_C
#endif

typedef int32_t WASMRawI32;
typedef uint32_t WASMRawU32;
typedef int64_t WASMRawI64;
typedef uint64_t WASMRawU64;
typedef float WASMRawF32;
typedef double WASMRawF64;

#include "common/wr_api_functions.def"

#ifdef EXTERN_C
}
#undef EXTERN_C
#endif

#undef WR_API_FUNCTION
