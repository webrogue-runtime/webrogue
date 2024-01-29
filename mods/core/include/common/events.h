#pragma once

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef enum webrogue_event_type {
    None = 0,
    Arrow,
    Char,
    Close,
    MouseLeftButtonPressed,
    MouseLeftButtonReleased,
    MouseMoved,
    Deadline,
    Console,
    Resize,
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
} webrogue_event;

typedef struct webrogue_raw_event {
    int32_t type;
    int32_t data1;
    int32_t data2;
    int32_t data3;
} webrogue_raw_event;

#ifdef __cplusplus
static_assert(sizeof(webrogue_raw_event) == 16,
              "invalid size of webrogue_raw_event");
#endif

#ifdef __cplusplus
}
#endif
