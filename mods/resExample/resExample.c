#include "../core/include/core.h"
#include "../core/include/macros.h"
#include "../core/include/wr_api.h"
#include <stdlib.h>
#include <string.h>

void resExampleInitializationStep() {
    const char *resName = "resExample/wrres/test_res.txt";
    int32_t rd = wr_res_open((WASMRawU64)resName, strlen(resName));
    char *data = malloc(wr_res_get_size(rd));
    wr_res_get_data(rd, (WASMRawU64)data);
    wr_res_close(rd);
    free(data);
}

WR_EXPORTED(void, init_mod_resExample)() {
    webrogue_core_add_initialization_step(resExampleInitializationStep);
}
