#include "../include/wrsqlite.h"
#include "../include/wr_api.h"
#include <cassert>
#include <cstddef>
#include <cstdint>
#include <cstring>
#include <vector>

extern "C" {
int32_t sqlite3wr_open_v2(const char *filename, /* Database filename (UTF-8) */
                          sqlite3wr *ppDb,      /* OUT: SQLite db handle */
                          int32_t flags,        /* Flags */
                          const char *zVfs      /* Name of VFS module to use */
) {
    *ppDb = (sqlite3wr)1;
    return SQLITE_OK;
}
int32_t sqlite3wr_close(sqlite3wr db) {
    return SQLITE_OK;
}
std::vector<char> wrsqliteErrmsg;
const char *sqlite3wr_errmsg(sqlite3wr db) {
    size_t size = wr_sqlite3_errmsg_size();
    wrsqliteErrmsg.resize(size + 1);
    wr_sqlite3_errmsg_get((int64_t)wrsqliteErrmsg.data());
    return wrsqliteErrmsg.data();
}
int32_t sqlite3wr_prepare_v2(
    sqlite3wr db,           /* Database handle */
    const char *zSql,       /* SQL statement, UTF-8 encoded */
    int32_t nByte,          /* Maximum length of zSql in bytes. */
    sqlite3wr_stmt *ppStmt, /* OUT: Statement handle */
    const char **pzTail     /* OUT: Pointer to unused portion of zSql */
) {
    return wr_sqlite3_prepare_v2((int64_t)zSql, nByte, (int64_t)ppStmt,
                                 (int64_t)pzTail);
}
int32_t sqlite3wr_step(sqlite3wr_stmt stmt) {
    return wr_sqlite3_step(stmt);
}
int64_t sqlite3wr_last_insert_rowid(sqlite3wr) {
    return wr_sqlite3_last_insert_rowid();
}
int32_t sqlite3wr_changes(sqlite3wr) {
    return wr_sqlite3_changes();
}
int32_t sqlite3wr_column_int(sqlite3wr_stmt stmt, int32_t iCol) {
    return wr_sqlite3_column_int(stmt, iCol);
}
int32_t sqlite3wr_column_type(sqlite3wr_stmt stmt, int32_t iCol) {
    return wr_sqlite3_column_type(stmt, iCol);
}
std::vector<unsigned char> lastText;
const unsigned char *sqlite3wr_column_text(sqlite3wr_stmt stmt, int32_t iCol) {
    int64_t len = wr_sqlite3_column_text_size(stmt, iCol);
    if (len < 0) {
        return nullptr;
    }
    if (lastText.size() < (len + 1)) {
        lastText.resize(len + 1);
    }
    wr_sqlite3_column_text_get(stmt, iCol, (int64_t)lastText.data());
    return lastText.data();
}
double sqlite3wr_column_double(sqlite3wr_stmt stmt, int32_t iCol) {
    return wr_sqlite3_column_double(stmt, iCol);
}
int64_t sqlite3wr_column_int64(sqlite3wr_stmt stmt, int32_t iCol) {
    return wr_sqlite3_column_int64(stmt, iCol);
}
int32_t sqlite3wr_column_bytes(sqlite3wr_stmt stmt, int32_t iCol) {
    return wr_sqlite3_column_bytes(stmt, iCol);
}
std::vector<unsigned char> lastBlob;
const void *sqlite3wr_column_blob(sqlite3wr_stmt stmt, int32_t iCol) {
    size_t len = sqlite3wr_column_bytes(stmt, iCol);
    if (lastBlob.size() < len) {
        lastBlob.resize(len);
    }
    wr_sqlite3_column_blob_get(stmt, iCol, (int64_t)lastBlob.data());
    return lastBlob.data();
}
int32_t sqlite3wr_finalize(sqlite3wr_stmt pStmt) {
    return wr_sqlite3_finalize(pStmt);
}
int32_t sqlite3wr_reset(sqlite3wr_stmt pStmt) {
    return wr_sqlite3_reset(pStmt);
}
int32_t sqlite3wr_bind_int(sqlite3wr_stmt stmt, int a, int b) {
    return wr_sqlite3_bind_int(stmt, a, b);
}
int32_t sqlite3wr_bind_null(sqlite3wr_stmt stmt, int a) {
    return wr_sqlite3_bind_null(stmt, a);
}
int32_t sqlite3wr_bind_text(sqlite3wr_stmt stmt, int a, const char *b, int c,
                            void (*d)(void *)) {
    assert(d == 0);
    return wr_sqlite3_bind_text(stmt, a, (int64_t)b, c >= 0 ? c : strlen(b));
}
int32_t sqlite3wr_bind_double(sqlite3wr_stmt stmt, int a, double b) {
    return wr_sqlite3_bind_double(stmt, a, b);
}
int32_t sqlite3wr_bind_int64(sqlite3wr_stmt stmt, int a, int64_t b) {
    return wr_sqlite3_bind_int64(stmt, a, b);
}
int32_t sqlite3wr_bind_blob(sqlite3wr_stmt stmt, int a, const void *b,
                            int32_t n, void (*)(void *)) {
    return wr_sqlite3_bind_blob(stmt, a, (int64_t)b, n);
}
}