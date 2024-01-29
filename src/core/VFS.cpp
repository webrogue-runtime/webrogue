#include "VFS.hpp"
#include "Config.hpp"
#include "ResourceStorage.hpp"
#include <algorithm>
#include <cassert>
#include <cstdint>
#include <cstdlib>
#include <cstring>
#include <memory>
#include <utility>
#include <vector>

namespace webrogue {
namespace core {

bool VFS::FD::isPreopened() {
    return false;
}

bool VFS::FD::seek(long int offset, uint8_t whence, size_t &outPos) {
    return false;
}

bool VFS::FD::write(const uint8_t *dataToWrite, size_t dataSize) {
    return false;
}

bool VFS::FD::read(uint8_t *dataToRead, size_t dataSize, size_t &nread) {
    return false;
}

void VFS::FD::commit() {
}

bool VFS::FD::close() {
    return false;
}

VFS::FD::~FD() {
}

class BasicFD : public VFS::FD {
public:
    std::string getName() override {
        return "BasicFD";
    }
};
class PreopenedDirFD : public VFS::FD {
public:
    std::string dirName;
    PreopenedDirFD(std::string dirName) : dirName(dirName){};
    std::string getName() override {
        return dirName;
    }
    bool isPreopened() override {
        return true;
    }
};
class CommonFD : public VFS::FD {
public:
    VFS *vfs;
    std::string path;
    std::vector<uint8_t> data;
    size_t cursor = 0;

    CommonFD(VFS *vfs, std::string path, std::vector<uint8_t> &data)
        : vfs(vfs), path(path), data(data){};
    std::string getName() override {
        return path;
    }
    bool seek(long int offset, uint8_t whence, size_t &outPos) override {
        long int newCursor = 0;
        switch (whence) {
        case 0:
            newCursor = offset;
            break;
        case 1:
            newCursor = cursor + offset;
            break;
        case 2:
            newCursor = data.size() + offset;
            break;
        default:
            return false;
        }
        if (newCursor < 0 || newCursor > data.size())
            return false;
        outPos = cursor = newCursor;
        return true;
    }
    bool write(const uint8_t *dataToWrite, size_t dataSize) override {
        if (cursor + dataSize > data.size())
            data.resize(cursor + dataSize);
        memcpy((data.data() + cursor), dataToWrite, dataSize);
        cursor += dataSize;
        return true;
    }

    bool read(uint8_t *dataToRead, size_t dataSize, size_t &nread) override {
        size_t readSize = std::min(dataSize, data.size() - cursor);
        memcpy(dataToRead, (data.data() + cursor), readSize);
        cursor += readSize;
        nread = readSize;
        return true;
    }
    void commit() override {
        sqlite3_stmt *stmt = vfs->fileDataUpdateStmt;
        int result = sqlite3_reset(stmt);
        if (result != SQLITE_OK)
            abort();
        result = sqlite3_bind_text(stmt, 2, path.c_str(), path.size(),
                                   SQLITE_STATIC);
        if (result != SQLITE_OK)
            abort();
        result =
            sqlite3_bind_blob(stmt, 1, data.data() ? data.data() : (void *)1,
                              data.size(), SQLITE_STATIC);
        if (result != SQLITE_OK)
            abort();
        result = sqlite3_step(stmt);
        if (result != SQLITE_DONE)
            abort();
    }
    bool close() override {
        commit();
        return true;
    }
};
class InvalidFD : public VFS::FD {
public:
    std::string getName() override {
        return "invalid";
    }
};

VFS::VFS(ResourceStorage *resourceStorage, Config *config)
    : resourceStorage(resourceStorage), config(config) {
    fs.push_back(std::make_unique<BasicFD>());
    fs.push_back(std::make_unique<BasicFD>());
    fs.push_back(std::make_unique<BasicFD>());

    fs.push_back(std::make_unique<PreopenedDirFD>("./"));
    std::string dbpath;
    if (config->hasDataPath) {
        dbpath = config->dataPath + "/vfs.db";
    } else {
        dbpath = "memory";
    }

    int result = sqlite3_open_v2(
        dbpath.c_str(), &mSqlite,
        SQLITE_OPEN_READWRITE | SQLITE_OPEN_CREATE | SQLITE_OPEN_NOMUTEX |
            (dbpath == "memory" ? SQLITE_OPEN_MEMORY : 0),
        nullptr);
    if (result != SQLITE_OK)
        abort();
    sqlite3_stmt *createStmt;
    result = sqlite3_prepare_v2(mSqlite,
                                "CREATE TABLE IF NOT EXISTS files ("
                                "path TEXT PRIMARY KEY,"
                                "data BLOB NOT NULL"
                                ");",
                                -1, &createStmt, nullptr);
    if (result != SQLITE_OK)
        abort();
    result = sqlite3_step(createStmt);
    if (result != SQLITE_DONE)
        abort();
    result = sqlite3_finalize(createStmt);
    if (result != SQLITE_OK)
        abort();

    result =
        sqlite3_prepare_v2(mSqlite, "UPDATE files SET data = ? WHERE path = ?;",
                           -1, &fileDataUpdateStmt, nullptr);
    if (result != SQLITE_OK)
        abort();
    result = sqlite3_prepare_v2(
        mSqlite, "INSERT INTO files (path, data) VALUES (?, \"\");", -1,
        &insertEmptyFileStmt, nullptr);
    if (result != SQLITE_OK)
        abort();
    result =
        sqlite3_prepare_v2(mSqlite, "SELECT data FROM files WHERE path = ?;",
                           -1, &getFileDataStmt, nullptr);
    if (result != SQLITE_OK)
        abort();
}
bool VFS::open(std::string path, size_t &outFd, bool append) {
    outFd = fs.size();
    if (path.substr(0, 1) == "/")
        return false;

    path = path.substr(2);
    for (auto &f : fs)
        if (f->getName() == path) {
            assert(false);
            return false;
        }
    int result;
    sqlite3_reset(getFileDataStmt);
    result = sqlite3_bind_text(getFileDataStmt, 1, path.c_str(), path.size(),
                               SQLITE_STATIC);
    if (result != SQLITE_OK)
        abort();
    result = sqlite3_step(getFileDataStmt);
    bool foundFile;
    std::vector<uint8_t> fileData{};
    switch (result) {
    case SQLITE_DONE:
        foundFile = false;
        break;
    case SQLITE_ROW:
        foundFile = true;
        if (append) {
            fileData.resize(sqlite3_column_bytes(getFileDataStmt, 0));
            if (fileData.size())
                memcpy(fileData.data(), sqlite3_column_blob(getFileDataStmt, 0),
                       fileData.size());
        }
        break;
    default:
        abort();
    }
    if (!foundFile) {
        sqlite3_reset(insertEmptyFileStmt);
        result = sqlite3_bind_text(insertEmptyFileStmt, 1, path.c_str(),
                                   path.size(), SQLITE_STATIC);
        if (result != SQLITE_OK)
            abort();
        result = sqlite3_step(insertEmptyFileStmt);
        if (result != SQLITE_DONE)
            abort();
    }
    auto f = std::make_unique<CommonFD>(this, path, fileData);
    if (append) {
        size_t outPos;
        f->seek(0, 2, outPos);
    }
    fdsToUpdate.insert(outFd);
    fs.push_back(std::move(f));
    return true;
}
bool VFS::preopendDirName(size_t fd, std::string &outName) {
    if (fd < 0 || fd >= fs.size())
        return false;
    auto &f = fs[fd];
    if (!f->isPreopened())
        return false;
    outName = f->getName();
    return true;
}
bool VFS::seek(size_t fd, long int offset, uint8_t whence, size_t &outPos) {
    if (fd < 0 || fd >= fs.size())
        return false;
    return fs[fd]->seek(offset, whence, outPos);
}
bool VFS::write(size_t fd, uint8_t *data, size_t size) {
    if (fd < 0 || fd >= fs.size())
        return false;
    fdsToUpdate.insert(fd);
    return fs[fd]->write(data, size);
}

bool VFS::read(size_t fd, uint8_t *data, size_t size, size_t &nread) {
    if (fd < 0 || fd >= fs.size())
        return false;
    return fs[fd]->read(data, size, nread);
}

bool VFS::close(size_t fd) {
    if (fd < 0 || fd >= fs.size())
        return false;
    return fs[fd]->close();
}
void VFS::commit() {
    if (fdsToUpdate.empty())
        return;
    for (auto fd : fdsToUpdate)
        fs[fd]->commit();
    fdsToUpdate.clear();
}
VFS::~VFS() {
    int result = sqlite3_finalize(fileDataUpdateStmt);
    if (result != SQLITE_OK)
        abort();
    result = sqlite3_finalize(insertEmptyFileStmt);
    if (result != SQLITE_OK)
        abort();
    result = sqlite3_finalize(getFileDataStmt);
    if (result != SQLITE_OK)
        abort();
    result = sqlite3_close_v2(mSqlite);
    if (result != SQLITE_OK)
        abort();
}
} // namespace core
} // namespace webrogue
