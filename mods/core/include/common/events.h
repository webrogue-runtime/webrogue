#pragma once

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef enum webrogue_event_type {
    None = 0,
    Close,
    Resize,
    Stdin,
} webrogue_event_type;

typedef enum webrogue_arrow_direction {
    left = 0,
    right,
    up,
    down,
} webrogue_arrow_direction;

typedef struct webrogue_event {
    webrogue_event_type type;
    int32_t data1;
    int32_t data2;
    int32_t data3;
} webrogue_event;

#ifdef __cplusplus
static_assert(sizeof(webrogue_event) == 16, "invalid size of webrogue_event");
#endif

#ifdef __cplusplus
}
#endif
