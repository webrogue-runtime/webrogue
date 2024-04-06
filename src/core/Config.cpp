#include "Config.hpp"
#include <cstdint>
#include <cstring>
#include <memory>
#include <vector>

namespace webrogue {
namespace core {
Config::Config(std::string dataPath) : dataPath(dataPath){};

void Config::setModsDirPath(std::string modsDirPath) {
    this->modsDirPath = modsDirPath;
}
void Config::setModsData(const uint8_t *data, size_t size, std::string name,
                         bool copy) {
    if (copy) {
        auto copied = std::make_shared<std::vector<uint8_t>>();
        copied->resize(size);
        memcpy(copied->data(), data, size);
        data = copied->data();
        copiedModsData.push_back(copied);
    }
    modsData.push_back({data, size, name});
}

void Config::setDisplay(std::shared_ptr<Display> display) {
    this->display = display;
}

std::list<Config::WrmodData> Config::getModsData() const {
    return modsData;
}
std::string Config::getDataPath() const {
    return dataPath;
}
std::optional<std::string> Config::getModsDirPath() const {
    return modsDirPath;
}
std::shared_ptr<Display> Config::getDisplay() const {
    return display;
}
} // namespace core
} // namespace webrogue
