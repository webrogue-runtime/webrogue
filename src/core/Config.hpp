#pragma once

#include "Display.hpp"
#include <cstddef>
#include <cstdint>
#include <functional>
#include <list>
#include <memory>
#include <optional>
#include <string>
#include <vector>

namespace webrogue {
namespace core {
class Config {
public:
    struct WrmodData {
        const uint8_t *data;
        size_t size;
        std::string name;
    };

    Config(std::string dataPath);

    void setModsDirPath(std::string modsDirPath);
    void setModsData(const uint8_t *data, size_t size, std::string name,
                     bool copy);
    void setDisplay(std::shared_ptr<Display> display);

    std::list<WrmodData> getModsData() const;
    std::optional<std::string> getModsDirPath() const;
    std::string getDataPath() const;
    std::shared_ptr<Display> getDisplay() const;

private:
    std::string dataPath;
    std::optional<std::string> modsDirPath;
    std::list<WrmodData> modsData;
    std::list<std::shared_ptr<std::vector<uint8_t>>> copiedModsData;
    std::shared_ptr<Display> display;
};
} // namespace core
} // namespace webrogue
