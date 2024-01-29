#include "../core/Config.hpp"

#include "../common/stringize.hpp"

#include stringize(WEBROGUE_EWRMOD_LIST_HEADER)

void inline load_embedded_mods(webrogue::core::Config *config) {
#define mod_to_embed(name)                                                     \
    config->addWrmodData(name##_ewrmod, name##_ewrmod_size, stringize(name));
#include stringize(WEBROGUE_MOD_LIST_HEADER)
#undef mod_to_embed
}
