#include "Config.hpp"

namespace webrogue {
namespace core {
void Config::addWrmodData(const uint8_t *data, size_t size, std::string name) {
    mods.push_back({data, size, name});
}
void Config::setDataPath(std::string path) {
    hasDataPath = true;
    dataPath = path;
}
} // namespace core
} // namespace webrogue
