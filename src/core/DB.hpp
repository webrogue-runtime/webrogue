#pragma once

#include "../../external/sqlite_amb/sqlite3.h"
#include <cstdint>
#include <string>

namespace webrogue {
namespace core {
class DB {
    class Storage;
    Storage *storage;

public:
    DB(std::string dbPath);
    ~DB();
    sqlite3 *getDb();
    sqlite3_stmt *stmtById(int64_t stmtId);
    void stmtDelete(int64_t stmtId);
    sqlite3_stmt **stmtNew(int64_t *pStmtId);
};
} // namespace core
} // namespace webrogue
