#include "../../core/wasm_types.hpp"
#include "shared_api_object.hpp"
using namespace webrogue::core;
// clang-format off
extern "C" void wr_start_color() {
    return webrogue::runtimes::native::sharedApiObject->wr_start_color();
}

extern "C" int32_t wr_get_color_pairs_count() {
    return webrogue::runtimes::native::sharedApiObject->wr_get_color_pairs_count().get();
}

extern "C" int32_t wr_get_colors_count() {
    return webrogue::runtimes::native::sharedApiObject->wr_get_colors_count().get();
}

extern "C" void wr_set_color(int32_t color, int32_t r, int32_t g, int32_t b) {
    return webrogue::runtimes::native::sharedApiObject->wr_set_color(WASMRawI32::make(color), WASMRawI32::make(r), WASMRawI32::make(g), WASMRawI32::make(b));
}

extern "C" void wr_set_color_pair(int32_t color_pair, int32_t fg, int32_t bg) {
    return webrogue::runtimes::native::sharedApiObject->wr_set_color_pair(WASMRawI32::make(color_pair), WASMRawI32::make(fg), WASMRawI32::make(bg));
}

extern "C" void wr_set_deadline(int32_t ms) {
    return webrogue::runtimes::native::sharedApiObject->wr_set_deadline(WASMRawI32::make(ms));
}

extern "C" int32_t wr_interrupt() {
    return webrogue::runtimes::native::sharedApiObject->wr_interrupt().get();
}

extern "C" void wr_copy_events(uint64_t out_buff_offset, int64_t size) {
    return webrogue::runtimes::native::sharedApiObject->wr_copy_events(WASMRawU64::make(out_buff_offset), WASMRawI64::make(size));
}

extern "C" void wr_stdout_write(uint64_t in_buff_offset, int64_t size) {
    return webrogue::runtimes::native::sharedApiObject->wr_stdout_write(WASMRawU64::make(in_buff_offset), WASMRawI64::make(size));
}

extern "C" void wr_debug_print(uint64_t in_buff_offset, int64_t size) {
    return webrogue::runtimes::native::sharedApiObject->wr_debug_print(WASMRawU64::make(in_buff_offset), WASMRawI64::make(size));
}

extern "C" int64_t wr_sqlite3_errmsg_size() {
    return webrogue::runtimes::native::sharedApiObject->wr_sqlite3_errmsg_size().get();
}

extern "C" void wr_sqlite3_errmsg_get(uint64_t out_err_offset) {
    return webrogue::runtimes::native::sharedApiObject->wr_sqlite3_errmsg_get(WASMRawU64::make(out_err_offset));
}

extern "C" int32_t wr_sqlite3_prepare_v2(uint64_t in_zSql_offset, int64_t zSql_size, uint64_t out_ppStmt_offset, uint64_t out_pzTail_offset) {
    return webrogue::runtimes::native::sharedApiObject->wr_sqlite3_prepare_v2(WASMRawU64::make(in_zSql_offset), WASMRawI64::make(zSql_size), WASMRawU64::make(out_ppStmt_offset), WASMRawU64::make(out_pzTail_offset)).get();
}

extern "C" int32_t wr_sqlite3_step(int64_t stmt) {
    return webrogue::runtimes::native::sharedApiObject->wr_sqlite3_step(WASMRawI64::make(stmt)).get();
}

extern "C" int64_t wr_sqlite3_last_insert_rowid() {
    return webrogue::runtimes::native::sharedApiObject->wr_sqlite3_last_insert_rowid().get();
}

extern "C" int32_t wr_sqlite3_changes() {
    return webrogue::runtimes::native::sharedApiObject->wr_sqlite3_changes().get();
}

extern "C" int32_t wr_sqlite3_column_int(int64_t stmt, int32_t iCol) {
    return webrogue::runtimes::native::sharedApiObject->wr_sqlite3_column_int(WASMRawI64::make(stmt), WASMRawI32::make(iCol)).get();
}

extern "C" int32_t wr_sqlite3_column_type(int64_t stmt, int32_t iCol) {
    return webrogue::runtimes::native::sharedApiObject->wr_sqlite3_column_type(WASMRawI64::make(stmt), WASMRawI32::make(iCol)).get();
}

extern "C" int64_t wr_sqlite3_column_text_size(int64_t stmt, int32_t iCol) {
    return webrogue::runtimes::native::sharedApiObject->wr_sqlite3_column_text_size(WASMRawI64::make(stmt), WASMRawI32::make(iCol)).get();
}

extern "C" void wr_sqlite3_column_text_get(int64_t stmt, int32_t iCol, uint64_t out_result_offset) {
    return webrogue::runtimes::native::sharedApiObject->wr_sqlite3_column_text_get(WASMRawI64::make(stmt), WASMRawI32::make(iCol), WASMRawU64::make(out_result_offset));
}

extern "C" double wr_sqlite3_column_double(int64_t stmt, int32_t iCol) {
    return webrogue::runtimes::native::sharedApiObject->wr_sqlite3_column_double(WASMRawI64::make(stmt), WASMRawI32::make(iCol)).get();
}

extern "C" int64_t wr_sqlite3_column_int64(int64_t stmt, int32_t iCol) {
    return webrogue::runtimes::native::sharedApiObject->wr_sqlite3_column_int64(WASMRawI64::make(stmt), WASMRawI32::make(iCol)).get();
}

extern "C" int32_t wr_sqlite3_column_bytes(int64_t stmt, int32_t iCol) {
    return webrogue::runtimes::native::sharedApiObject->wr_sqlite3_column_bytes(WASMRawI64::make(stmt), WASMRawI32::make(iCol)).get();
}

extern "C" void wr_sqlite3_column_blob_get(int64_t stmt, int32_t iCol, uint64_t out_result_offset) {
    return webrogue::runtimes::native::sharedApiObject->wr_sqlite3_column_blob_get(WASMRawI64::make(stmt), WASMRawI32::make(iCol), WASMRawU64::make(out_result_offset));
}

extern "C" int32_t wr_sqlite3_finalize(int64_t stmt) {
    return webrogue::runtimes::native::sharedApiObject->wr_sqlite3_finalize(WASMRawI64::make(stmt)).get();
}

extern "C" int32_t wr_sqlite3_reset(int64_t stmt) {
    return webrogue::runtimes::native::sharedApiObject->wr_sqlite3_reset(WASMRawI64::make(stmt)).get();
}

extern "C" int32_t wr_sqlite3_bind_int(int64_t stmt, int32_t a, int32_t b) {
    return webrogue::runtimes::native::sharedApiObject->wr_sqlite3_bind_int(WASMRawI64::make(stmt), WASMRawI32::make(a), WASMRawI32::make(b)).get();
}

extern "C" int32_t wr_sqlite3_bind_null(int64_t stmt, int32_t a) {
    return webrogue::runtimes::native::sharedApiObject->wr_sqlite3_bind_null(WASMRawI64::make(stmt), WASMRawI32::make(a)).get();
}

extern "C" int32_t wr_sqlite3_bind_text(int64_t stmt, int32_t a, uint64_t in_text_offset, int64_t b_size) {
    return webrogue::runtimes::native::sharedApiObject->wr_sqlite3_bind_text(WASMRawI64::make(stmt), WASMRawI32::make(a), WASMRawU64::make(in_text_offset), WASMRawI64::make(b_size)).get();
}

extern "C" int32_t wr_sqlite3_bind_double(int64_t stmt, int32_t a, double b) {
    return webrogue::runtimes::native::sharedApiObject->wr_sqlite3_bind_double(WASMRawI64::make(stmt), WASMRawI32::make(a), WASMRawF64::make(b)).get();
}

extern "C" int32_t wr_sqlite3_bind_int64(int64_t stmt, int32_t a, int64_t b) {
    return webrogue::runtimes::native::sharedApiObject->wr_sqlite3_bind_int64(WASMRawI64::make(stmt), WASMRawI32::make(a), WASMRawI64::make(b)).get();
}

extern "C" int32_t wr_sqlite3_bind_blob(int64_t stmt, int32_t a, uint64_t in_blob_offset, int32_t n) {
    return webrogue::runtimes::native::sharedApiObject->wr_sqlite3_bind_blob(WASMRawI64::make(stmt), WASMRawI32::make(a), WASMRawU64::make(in_blob_offset), WASMRawI32::make(n)).get();
}

extern "C" uint32_t wr_res_open(uint64_t name, uint32_t nameLen) {
    return webrogue::runtimes::native::sharedApiObject->wr_res_open(WASMRawU64::make(name), WASMRawU32::make(nameLen)).get();
}

extern "C" uint64_t wr_res_get_size(uint32_t rd) {
    return webrogue::runtimes::native::sharedApiObject->wr_res_get_size(WASMRawU32::make(rd)).get();
}

extern "C" void wr_res_get_data(uint32_t rd, uint64_t outData) {
    return webrogue::runtimes::native::sharedApiObject->wr_res_get_data(WASMRawU32::make(rd), WASMRawU64::make(outData));
}

extern "C" void wr_res_close(uint32_t rd) {
    return webrogue::runtimes::native::sharedApiObject->wr_res_close(WASMRawU32::make(rd));
}

extern "C" void initWrNativeApi() {}
