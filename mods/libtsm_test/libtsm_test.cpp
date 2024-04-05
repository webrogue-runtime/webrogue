#include "../core/include/core.h"
#include "../core/include/macros.h"
#include "../core/include/wr_api.h"
#include <cstdlib>
#include <string>

void llibtsm_testInitializationStep() {
    std::string str = "str!";
    wr_stdout_write((WASMRawU64)str.c_str(), str.size());
    while (true) {
        webrogue_core_interrupt();
        size_t event_count;
        webrogue_event const *events = webrogue_core_get_events(&event_count);
        for (int i = 0; i < event_count; i++) {
            webrogue_event event = events[i];
            if (event.type == webrogue_event_type::Close)
                exit(0);
        }
    }
}

extern "C" WR_EXPORTED(void, init_mod_libtsm_test)() {
    webrogue_core_add_initialization_step(llibtsm_testInitializationStep);
}
