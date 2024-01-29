#pragma once

#include "../../external/sqlite_amb/sqlite3.h"
#include "Config.hpp"
#include "ResourceStorage.hpp"
#include <cstddef>
#include <cstdint>
#include <memory>
#include <set>
#include <string>
#include <vector>

namespace webrogue {
namespace core {
class VFS {
public:
    class FD {
    public:
        virtual std::string getName() = 0;
        virtual bool seek(long int offset, uint8_t whence, size_t &outPos);
        virtual bool write(const uint8_t *dataToWrite, size_t dataSize);
        virtual bool read(uint8_t *dataToRead, size_t dataSize, size_t &nread);
        virtual bool isPreopened();
        virtual void commit();
        virtual bool close();
        virtual ~FD();
    };
    sqlite3 *mSqlite;
    sqlite3_stmt *fileDataUpdateStmt;
    sqlite3_stmt *insertEmptyFileStmt;
    sqlite3_stmt *getFileDataStmt;
    std::set<size_t> fdsToUpdate;
    std::vector<std::unique_ptr<FD>> fs;
    ResourceStorage *resourceStorage;
    Config *config;
    VFS(ResourceStorage *resourceStorage, Config *config);
    bool open(std::string path, size_t &outFd, bool append);
    bool preopendDirName(size_t fd, std::string &outName);
    bool seek(size_t fd, long int offset, uint8_t whence, size_t &outPos);
    bool write(size_t fd, uint8_t *data, size_t size);
    bool read(size_t fd, uint8_t *data, size_t size, size_t &nread);
    bool close(size_t fd);
    void commit();
    ~VFS();
};
} // namespace core
} // namespace webrogue
