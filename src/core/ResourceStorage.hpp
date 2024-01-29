#pragma once

#include "ConsoleStream.hpp"
#include <cstdint>
#include <functional>
#include <map>
#include <set>
#include <string>
#include <vector>
namespace webrogue {
namespace core {
class ResourceStorage {
public:
    std::map<std::string, std::vector<uint8_t>> filemap;
    ConsoleStream *wrout;
    ConsoleStream *wrerr;
    std::function<void()> interrupt = []() {
    };

public:
    std::string dataPath;
    std::set<std::string> modNames;
    bool hasFile(std::string path);
    std::vector<uint8_t> &getFile(std::string path);
    void addDirectory(std::string modName, std::string path);
    void addWrmodData(std::string modName, const uint8_t *data, size_t size);
    bool loadDir(std::string path, std::string name);
    bool loadWrmodData(const uint8_t *data, size_t size, std::string name);
    bool loadWrmodFile(std::string path, std::string name);

    ResourceStorage(ConsoleStream *pNrout, ConsoleStream *pNrerr);

private:
    bool readRealFile(std::vector<uint8_t> &out, std::string path);
    void traverseDirectory(std::string modName, std::string rootPath,
                           std::string extraPath);
    void loadFile(std::string modName, std::string realPath,
                  std::string extraPath);
};
} // namespace core
} // namespace webrogue
