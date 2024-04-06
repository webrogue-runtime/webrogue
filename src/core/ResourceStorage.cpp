// it looks like windef.h header contains min macro
#define NOMINMAX

#include "ResourceStorage.hpp"
#include "../../external/zstd/zstd.h"
#include "sys/stat.h"
#include "xz.h"
#include <algorithm>
#include <cstddef>
#include <cstdint>
#include <cstdlib>
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
ResourceStorage::ResourceStorage() {
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
        std::string const name(drnt->d_name);
        if (name == curDir || name == parDir)
            continue;
        struct stat s;
        bool const isDir = s.st_mode & S_IFDIR;
        std::string const newExtraPath = extraPath + "/" + name;
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
    std::vector<uint8_t> &fileData = fileMap[modName + extraPath] = {};
    readRealFile(fileData, realPath);
}
std::vector<uint8_t> &ResourceStorage::getFile(std::string path) {
    return fileMap[path];
}
bool ResourceStorage::hasFile(std::string path) const {
    return fileMap.count(path);
}

void ResourceStorage::addWrmodData(std::string modName, const uint8_t *data,
                                   size_t size) {
    if (modNames.count(modName))
        return;
    modNames.insert(modName);
    size_t const rModnameLen = strnlen((const char *)data, 128);
    std::string const rModname{(const char *)data, rModnameLen};
    if (rModname != modName)
        return;
    data += rModnameLen + 1;
    size -= rModnameLen + 1;
    if (rModname != modName)
        return;

    size_t const rCompressorNameLen = strnlen((const char *)data, 128);
    std::string const rCompressorName{(const char *)data, rCompressorNameLen};
    data += rCompressorNameLen + 1;
    size -= rCompressorNameLen + 1;

    std::string const hexSize{(const char *)data, 16};
    const size_t decompressedSize = std::stoul(hexSize, nullptr, 16);
    data += 17;
    size -= 17;
    std::vector<uint8_t> decompressedData;
    decompressedData.resize(decompressedSize + 1);
    decompressedData[decompressedSize] = '\0'; // guard
    std::list<DecompressedFilePointer> decompressedFiles;

    if (rCompressorName == "xz")
        decompressXZ(data, size, decompressedData, decompressedSize,
                     decompressedFiles);
    else if (rCompressorName == "zstd")
        decompressZstd(data, size, decompressedData, decompressedSize,
                       decompressedFiles);
    else if (rCompressorName == "raw")
        memcpy(decompressedData.data(), data, size);
    else
        return;

    size_t cursor = 0;
    while (cursor < decompressedSize) {
        size_t const nameLen = std::strlen((char *)&decompressedData[cursor]);
        if (cursor + nameLen >= decompressedSize)
            return;
        std::string const filename{(char *)&decompressedData[cursor], nameLen};
        cursor += nameLen + 1;
        if (cursor + 17 >= decompressedSize)
            return;
        unsigned int const fileSize =
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
        fileMap[modName + "/" + pointer.filename] = {
            &decompressedData[pointer.cursor],
            &decompressedData[pointer.cursor] + pointer.size};
    }
}

void ResourceStorage::decompressXZ(
    const uint8_t *data, size_t size, std::vector<uint8_t> &decompressedData,
    size_t decompressedSize,
    std::list<DecompressedFilePointer> &decompressedFiles) {
    size_t const streamSize = 64 * 1024;
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
}

void ResourceStorage::decompressZstd(
    const uint8_t *data, size_t size, std::vector<uint8_t> &decompressedData,
    size_t decompressedSize,
    std::list<DecompressedFilePointer> &decompressedFiles) {

    size_t const buffInSize = ZSTD_DStreamInSize();
    std::vector<char> buffIn;
    buffIn.resize(buffInSize);
    size_t buffInOffset = 0;

    size_t const buffOutSize = ZSTD_DStreamOutSize();
    std::vector<char> buffOut;
    buffOut.resize(buffOutSize);
    size_t buffOutOffset = 0;

    ZSTD_DCtx *const dctx = ZSTD_createDCtx();

    while (size_t const read = std::min(size - buffInOffset, buffInSize)) {
        ZSTD_inBuffer input = {data + buffInOffset, read, 0};
        buffInOffset += read;
        while (input.pos < input.size) {
            ZSTD_outBuffer output = {buffOut.data(), buffOutSize, 0};
            size_t const ret = ZSTD_decompressStream(dctx, &output, &input);
            if (ZSTD_isError(ret)) {
                // *wrout << ZSTD_getErrorName(ret) << "\n";
                abort();
            }
            memcpy(decompressedData.data() + buffOutOffset, buffOut.data(),
                   output.pos);
            buffOutOffset += output.pos;
        }
        interrupt();
    }
}

bool ResourceStorage::loadDir(std::string path, std::string name) {
    if (modNames.count(name))
        return true;
    // *wrout << "loading directory \"" << name << "\"...\n";
    addDirectory(name, path);
    return true;
}

bool ResourceStorage::loadWrmodData(const uint8_t *data, size_t size,
                                    std::string name) {
    if (modNames.count(name))
        return true;
    // *wrout << "loading \"" << name << "\" from memory...\n";
    addWrmodData(name, data, size);
    return true;
}

bool ResourceStorage::readRealFile(std::vector<uint8_t> &out,
                                   std::string path) {
    std::ifstream file(path, std::ios::in | std::ios::binary);
    if (!file.is_open())
        return false;
    file.seekg(0, std::ios::end);
    size_t const length = file.tellg();
    file.seekg(0, std::ios::beg);
    out.resize(length);
    file.read(reinterpret_cast<char *>(out.data()), length);
    return true;
}

bool ResourceStorage::loadWrmodFile(std::string path, std::string name) {
    if (modNames.count(name))
        return true;
    // *wrout << "loading file \"" << path << "\"...\n";
    std::vector<uint8_t> compressedData;
    readRealFile(compressedData, path);
    addWrmodData(name, compressedData.data(), compressedData.size());
    return true;
}

} // namespace core
} // namespace webrogue
