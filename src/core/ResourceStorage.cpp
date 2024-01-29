#include "ResourceStorage.hpp"
#include "sys/stat.h"
#include "xz.h"
#include <cstddef>
#include <cstdint>
#include <cstring>
#include <dirent.h>
#include <fstream>
#include <iostream>
#include <iterator>
#include <list>
#include <string>
#include <vector>

namespace webrogue {
namespace core {
ResourceStorage::ResourceStorage(ConsoleStream *pNrout, ConsoleStream *pNrerr)
    : wrout(pNrout), wrerr(pNrerr) {
}

void ResourceStorage::addDirectory(std::string modName, std::string path) {
    struct stat info;
    if (stat((path + "/mod.a").c_str(), &info))
        return;
    if (modNames.count(modName))
        return;
    modNames.insert(modName);
    traverseDirectory(modName, path, "");
}
void ResourceStorage::traverseDirectory(std::string modName,
                                        std::string rootPath,
                                        std::string extraPath) {
    static const std::string curDir = ".";
    static const std::string parDir = "..";
    DIR *dir;
    struct dirent *drnt;
    dir = opendir((rootPath + extraPath).c_str());
    if (!dir)
        return;

    while ((drnt = readdir(dir)) != NULL) {
        std::string name(drnt->d_name);
        if (name == curDir || name == parDir)
            continue;
        struct stat s;
        bool isDir = s.st_mode & S_IFDIR;
        std::string newExtraPath = extraPath + "/" + name;
        if (stat((rootPath + newExtraPath).c_str(), &s) != 0)
            continue;

        if (isDir) {
            traverseDirectory(modName, rootPath, newExtraPath);
        } else {
            loadFile(modName, rootPath + newExtraPath, newExtraPath);
        }
    }

    closedir(dir);
}
void ResourceStorage::loadFile(std::string modName, std::string realPath,
                               std::string extraPath) {
    std::vector<uint8_t> &fileData = filemap[modName + extraPath] = {};
    readRealFile(fileData, realPath);
}
std::vector<uint8_t> &ResourceStorage::getFile(std::string path) {
    return filemap[path];
}
bool ResourceStorage::hasFile(std::string path) {
    return filemap.count(path);
}

struct DecompressedFilePointer {
    size_t cursor;
    size_t size;
    std::string filename;
};

void ResourceStorage::addWrmodData(std::string modName, const uint8_t *data,
                                   size_t size) {
    if (modNames.count(modName))
        return;
    modNames.insert(modName);
    size_t readedModnameLen = strnlen((const char *)data, 128);
    std::string readedModname{(const char *)data, readedModnameLen};
    if (readedModname != modName)
        return;
    data += readedModnameLen + 1;
    size -= readedModnameLen + 1;
    std::string hexSize{(const char *)data, 16};
    unsigned int decompressedSize = std::stoul(hexSize, nullptr, 16);
    data += 17;
    size -= 17;
    std::vector<uint8_t> decompressedData;
    decompressedData.resize(decompressedSize + 1);
    decompressedData[decompressedSize] = '\0'; // guard
    size_t streamSize = 64 * 1024;
    struct xz_buf b;
    b.in = data;
    b.in_pos = 0;
    b.in_size = size;
    b.out = decompressedData.data();
    b.out_pos = 0;
    b.out_size = streamSize;
    xz_crc32_init();
    struct xz_dec *s = xz_dec_init(XZ_PREALLOC, 1 << 26);
    enum xz_ret ret;
    while (b.out < decompressedData.data() + decompressedSize) {
        ret = xz_dec_run(s, &b);
        if (ret != XZ_STREAM_END && ret != XZ_OK)
            return;
        b.out += b.out_pos;
        b.out_pos = 0;
        interrupt();
    }
    xz_dec_end(s);
    if (ret != XZ_STREAM_END) {
        return;
    }
    std::list<DecompressedFilePointer> decompressedFiles;
    size_t cursor = 0;
    while (cursor < decompressedSize) {
        size_t nameLen = std::strlen((char *)&decompressedData[cursor]);
        if (cursor + nameLen >= decompressedSize)
            return;
        std::string filename{(char *)&decompressedData[cursor], nameLen};
        cursor += nameLen + 1;
        if (cursor + 17 >= decompressedSize)
            return;
        unsigned int fileSize =
            std::stoul({(char *)&decompressedData[cursor], 16}, nullptr, 16);
        cursor += 17;
        if (cursor + fileSize > decompressedSize)
            return;
        decompressedFiles.push_back({cursor, fileSize, filename});
        cursor += fileSize;
    }
    if (cursor != decompressedSize) {
        return; // overread, but how?
    }
    for (auto pointer : decompressedFiles) {
        filemap[modName + "/" + pointer.filename] = {
            &decompressedData[pointer.cursor],
            &decompressedData[pointer.cursor] + pointer.size};
    }
}

bool ResourceStorage::loadDir(std::string path, std::string name) {
    if (modNames.count(name))
        return true;
    *wrout << "loading directory \"" << name << "\"...\n";
    addDirectory(name, path);
    return true;
}

bool ResourceStorage::loadWrmodData(const uint8_t *data, size_t size,
                                    std::string name) {
    if (modNames.count(name))
        return true;
    *wrout << "loading \"" << name << "\" from memory...\n";
    addWrmodData(name, data, size);
    return true;
}

bool ResourceStorage::readRealFile(std::vector<uint8_t> &out,
                                   std::string path) {
    std::ifstream file(path, std::ios::in | std::ios::binary);
    if (!file.is_open())
        return false;
    file.unsetf(std::ios::skipws);
    file.seekg(0, std::ios_base::end);
    size_t fileSize = file.tellg();
    file.seekg(0, std::ios_base::beg);
    out.resize(0);
    out.reserve(fileSize);
    out.insert(out.begin(), std::istream_iterator<uint8_t>(file),
               std::istream_iterator<uint8_t>());
    return true;
}

bool ResourceStorage::loadWrmodFile(std::string path, std::string name) {
    if (modNames.count(name))
        return true;
    *wrout << "loading file \"" << path << "\"...\n";
    std::vector<uint8_t> compressedData;
    readRealFile(compressedData, path);
    addWrmodData(name, compressedData.data(), compressedData.size());
    return true;
}

} // namespace core
} // namespace webrogue
