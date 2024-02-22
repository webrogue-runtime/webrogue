#pragma once

#include <cstddef>
#include <cstdint>
#include <functional>
#include <list>
#include <optional>
#include <string>

namespace webrogue {
namespace core {
class Config {
public:
    bool hasDataPath = false;
    bool endOutputOnExit = true;
    std::string dataPath;
    std::string modsPath;
    bool hasModsPath = false;
    std::function<void()> onFrameEnd = []() {
    };
    bool loadsModsFromDataPath = false;

    // data should not be freed before Dispatcher is initialized;
    struct WrmodData {
        const uint8_t *data;
        size_t size;
        std::string name;
    };
    std::list<WrmodData> mods;
    void addWrmodData(const uint8_t *data, size_t size, std::string name);
    void setDataPath(std::string path);
    void setModsPath(std::string path);
};
} // namespace core
} // namespace webrogue
