#pragma once

#include "common/colors.h"
#include "common/events.h"

#ifdef __cplusplus
extern "C" {
#endif

#include <stddef.h>
#include <stdint.h>

typedef void (*webrogue_initialization_step)();
void webrogue_core_add_initialization_step(
    webrogue_initialization_step observer);

void webrogue_core_print(const char *);

#ifdef __cplusplus
}
#endif
