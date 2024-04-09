#include "../core/include/core.h"
#include "../core/include/macros.h"
#include "../core/include/wr_api.h"
#include <cmath>
#include <cstdint>
#include <cstdlib>
#include <string>
#include <vector>

void llibtsm_testInitializationStep() {
    int i = 0;
    while (true) {
        std::string str = std::to_string(i);
        int spacesCount = 5 + 3 * std::sin(static_cast<float>(i) / 3.1415 / 5);
        spacesCount = 3;
        for (int j = 0; j < spacesCount; j++) {
            str += " ";
        }
        // str = str + str + str + str;
        i++;
        wr_stdout_write((WASMRawU64)str.c_str(), str.size());
        uint32_t const eventCount = wr_poll(0);
        std::vector<webrogue_event> events;
        events.resize(eventCount);
        wr_copy_events((WASMRawU64)events.data(), eventCount);

        for (webrogue_event const event : events) {
            if (event.type == webrogue_event_type::Close)
                return;
        }
    }
}

extern "C" WR_EXPORTED(void, init_mod_libtsm_test)() {
    webrogue_core_add_initialization_step(llibtsm_testInitializationStep);
}
