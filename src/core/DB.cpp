#include "DB.hpp"

#include "../../external/sqlite_amb/sqlite3.h"
#include <cstdint>
#include <iostream>
#include <map>
#include <stack>
#include <stdexcept>
#include <vector>

namespace webrogue {
namespace core {
class DB::Storage {
public:
    sqlite3 *mSqlite;
    std::vector<sqlite3_stmt *> stmts;
    std::stack<int64_t> finalizedIds;
};

DB::DB(std::string dbPath) {
    storage = new DB::Storage();
    int result = sqlite3_open_v2(
        dbPath.c_str(), &storage->mSqlite,
        SQLITE_OPEN_READWRITE | SQLITE_OPEN_CREATE | SQLITE_OPEN_NOMUTEX |
            (dbPath == "memory" ? SQLITE_OPEN_MEMORY : 0),
        nullptr);
    if (result != SQLITE_OK)
        throw std::runtime_error(sqlite3_errmsg(storage->mSqlite));
}
DB::~DB() {
    sqlite3_close_v2(storage->mSqlite);
    delete storage;
}
sqlite3_stmt *DB::stmtById(int64_t stmtId) {
    return storage->stmts[stmtId - 10];
}
void DB::stmtDelete(int64_t stmtId) {
    storage->stmts[stmtId - 10] = nullptr;
    storage->finalizedIds.push(stmtId - 10);
}
sqlite3_stmt **DB::stmtNew(int64_t *pStmtId) {
    int64_t id;
    if (storage->finalizedIds.empty()) {
        storage->stmts.push_back(nullptr);
        id = storage->stmts.size() - 1;
    } else {
        id = storage->finalizedIds.top();
        storage->finalizedIds.pop();
    }
    *pStmtId = id + 10;
    return &storage->stmts[id];
}
sqlite3 *DB::getDb() {
    return storage->mSqlite;
}

} // namespace core
} // namespace webrogue
