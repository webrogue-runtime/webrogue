#include "wasm2c_runtime.hpp"
using namespace webrogue::core;

// clang-format off
extern "C" {
u32 w2c_webrogue_wr_poll(struct w2c_webrogue *env, u32 timeout_ms) {
    auto runtime =
        ((webrogue::runtimes::wasm2c::Wasm2cModsRuntime *)env);
    return runtime->apiObject.wr_poll(WASMRawI32::make(timeout_ms)).get();
}

void w2c_webrogue_wr_copy_events(struct w2c_webrogue *env, u64 out_buff_offset, u64 size) {
    auto runtime =
        ((webrogue::runtimes::wasm2c::Wasm2cModsRuntime *)env);
    return runtime->apiObject.wr_copy_events(WASMRawU64::make(out_buff_offset), WASMRawI64::make(size));
}

void w2c_webrogue_wr_stdout_write(struct w2c_webrogue *env, u64 in_buff_offset, u64 size) {
    auto runtime =
        ((webrogue::runtimes::wasm2c::Wasm2cModsRuntime *)env);
    return runtime->apiObject.wr_stdout_write(WASMRawU64::make(in_buff_offset), WASMRawI64::make(size));
}

void w2c_webrogue_wr_debug_print(struct w2c_webrogue *env, u64 in_buff_offset, u64 size) {
    auto runtime =
        ((webrogue::runtimes::wasm2c::Wasm2cModsRuntime *)env);
    return runtime->apiObject.wr_debug_print(WASMRawU64::make(in_buff_offset), WASMRawI64::make(size));
}

u64 w2c_webrogue_wr_sqlite3_errmsg_size(struct w2c_webrogue *env) {
    auto runtime =
        ((webrogue::runtimes::wasm2c::Wasm2cModsRuntime *)env);
    return runtime->apiObject.wr_sqlite3_errmsg_size().get();
}

void w2c_webrogue_wr_sqlite3_errmsg_get(struct w2c_webrogue *env, u64 out_err_offset) {
    auto runtime =
        ((webrogue::runtimes::wasm2c::Wasm2cModsRuntime *)env);
    return runtime->apiObject.wr_sqlite3_errmsg_get(WASMRawU64::make(out_err_offset));
}

u32 w2c_webrogue_wr_sqlite3_prepare_v2(struct w2c_webrogue *env, u64 in_zSql_offset, u64 zSql_size, u64 out_ppStmt_offset, u64 out_pzTail_offset) {
    auto runtime =
        ((webrogue::runtimes::wasm2c::Wasm2cModsRuntime *)env);
    return runtime->apiObject.wr_sqlite3_prepare_v2(WASMRawU64::make(in_zSql_offset), WASMRawI64::make(zSql_size), WASMRawU64::make(out_ppStmt_offset), WASMRawU64::make(out_pzTail_offset)).get();
}

u32 w2c_webrogue_wr_sqlite3_step(struct w2c_webrogue *env, u64 stmt) {
    auto runtime =
        ((webrogue::runtimes::wasm2c::Wasm2cModsRuntime *)env);
    return runtime->apiObject.wr_sqlite3_step(WASMRawI64::make(stmt)).get();
}

u64 w2c_webrogue_wr_sqlite3_last_insert_rowid(struct w2c_webrogue *env) {
    auto runtime =
        ((webrogue::runtimes::wasm2c::Wasm2cModsRuntime *)env);
    return runtime->apiObject.wr_sqlite3_last_insert_rowid().get();
}

u32 w2c_webrogue_wr_sqlite3_changes(struct w2c_webrogue *env) {
    auto runtime =
        ((webrogue::runtimes::wasm2c::Wasm2cModsRuntime *)env);
    return runtime->apiObject.wr_sqlite3_changes().get();
}

u32 w2c_webrogue_wr_sqlite3_column_int(struct w2c_webrogue *env, u64 stmt, u32 iCol) {
    auto runtime =
        ((webrogue::runtimes::wasm2c::Wasm2cModsRuntime *)env);
    return runtime->apiObject.wr_sqlite3_column_int(WASMRawI64::make(stmt), WASMRawI32::make(iCol)).get();
}

u32 w2c_webrogue_wr_sqlite3_column_type(struct w2c_webrogue *env, u64 stmt, u32 iCol) {
    auto runtime =
        ((webrogue::runtimes::wasm2c::Wasm2cModsRuntime *)env);
    return runtime->apiObject.wr_sqlite3_column_type(WASMRawI64::make(stmt), WASMRawI32::make(iCol)).get();
}

u64 w2c_webrogue_wr_sqlite3_column_text_size(struct w2c_webrogue *env, u64 stmt, u32 iCol) {
    auto runtime =
        ((webrogue::runtimes::wasm2c::Wasm2cModsRuntime *)env);
    return runtime->apiObject.wr_sqlite3_column_text_size(WASMRawI64::make(stmt), WASMRawI32::make(iCol)).get();
}

void w2c_webrogue_wr_sqlite3_column_text_get(struct w2c_webrogue *env, u64 stmt, u32 iCol, u64 out_result_offset) {
    auto runtime =
        ((webrogue::runtimes::wasm2c::Wasm2cModsRuntime *)env);
    return runtime->apiObject.wr_sqlite3_column_text_get(WASMRawI64::make(stmt), WASMRawI32::make(iCol), WASMRawU64::make(out_result_offset));
}

double w2c_webrogue_wr_sqlite3_column_double(struct w2c_webrogue *env, u64 stmt, u32 iCol) {
    auto runtime =
        ((webrogue::runtimes::wasm2c::Wasm2cModsRuntime *)env);
    return runtime->apiObject.wr_sqlite3_column_double(WASMRawI64::make(stmt), WASMRawI32::make(iCol)).get();
}

u64 w2c_webrogue_wr_sqlite3_column_int64(struct w2c_webrogue *env, u64 stmt, u32 iCol) {
    auto runtime =
        ((webrogue::runtimes::wasm2c::Wasm2cModsRuntime *)env);
    return runtime->apiObject.wr_sqlite3_column_int64(WASMRawI64::make(stmt), WASMRawI32::make(iCol)).get();
}

u32 w2c_webrogue_wr_sqlite3_column_bytes(struct w2c_webrogue *env, u64 stmt, u32 iCol) {
    auto runtime =
        ((webrogue::runtimes::wasm2c::Wasm2cModsRuntime *)env);
    return runtime->apiObject.wr_sqlite3_column_bytes(WASMRawI64::make(stmt), WASMRawI32::make(iCol)).get();
}

void w2c_webrogue_wr_sqlite3_column_blob_get(struct w2c_webrogue *env, u64 stmt, u32 iCol, u64 out_result_offset) {
    auto runtime =
        ((webrogue::runtimes::wasm2c::Wasm2cModsRuntime *)env);
    return runtime->apiObject.wr_sqlite3_column_blob_get(WASMRawI64::make(stmt), WASMRawI32::make(iCol), WASMRawU64::make(out_result_offset));
}

u32 w2c_webrogue_wr_sqlite3_finalize(struct w2c_webrogue *env, u64 stmt) {
    auto runtime =
        ((webrogue::runtimes::wasm2c::Wasm2cModsRuntime *)env);
    return runtime->apiObject.wr_sqlite3_finalize(WASMRawI64::make(stmt)).get();
}

u32 w2c_webrogue_wr_sqlite3_reset(struct w2c_webrogue *env, u64 stmt) {
    auto runtime =
        ((webrogue::runtimes::wasm2c::Wasm2cModsRuntime *)env);
    return runtime->apiObject.wr_sqlite3_reset(WASMRawI64::make(stmt)).get();
}

u32 w2c_webrogue_wr_sqlite3_bind_int(struct w2c_webrogue *env, u64 stmt, u32 a, u32 b) {
    auto runtime =
        ((webrogue::runtimes::wasm2c::Wasm2cModsRuntime *)env);
    return runtime->apiObject.wr_sqlite3_bind_int(WASMRawI64::make(stmt), WASMRawI32::make(a), WASMRawI32::make(b)).get();
}

u32 w2c_webrogue_wr_sqlite3_bind_null(struct w2c_webrogue *env, u64 stmt, u32 a) {
    auto runtime =
        ((webrogue::runtimes::wasm2c::Wasm2cModsRuntime *)env);
    return runtime->apiObject.wr_sqlite3_bind_null(WASMRawI64::make(stmt), WASMRawI32::make(a)).get();
}

u32 w2c_webrogue_wr_sqlite3_bind_text(struct w2c_webrogue *env, u64 stmt, u32 a, u64 in_text_offset, u64 b_size) {
    auto runtime =
        ((webrogue::runtimes::wasm2c::Wasm2cModsRuntime *)env);
    return runtime->apiObject.wr_sqlite3_bind_text(WASMRawI64::make(stmt), WASMRawI32::make(a), WASMRawU64::make(in_text_offset), WASMRawI64::make(b_size)).get();
}

u32 w2c_webrogue_wr_sqlite3_bind_double(struct w2c_webrogue *env, u64 stmt, u32 a, double b) {
    auto runtime =
        ((webrogue::runtimes::wasm2c::Wasm2cModsRuntime *)env);
    return runtime->apiObject.wr_sqlite3_bind_double(WASMRawI64::make(stmt), WASMRawI32::make(a), WASMRawF64::make(b)).get();
}

u32 w2c_webrogue_wr_sqlite3_bind_int64(struct w2c_webrogue *env, u64 stmt, u32 a, u64 b) {
    auto runtime =
        ((webrogue::runtimes::wasm2c::Wasm2cModsRuntime *)env);
    return runtime->apiObject.wr_sqlite3_bind_int64(WASMRawI64::make(stmt), WASMRawI32::make(a), WASMRawI64::make(b)).get();
}

u32 w2c_webrogue_wr_sqlite3_bind_blob(struct w2c_webrogue *env, u64 stmt, u32 a, u64 in_blob_offset, u32 n) {
    auto runtime =
        ((webrogue::runtimes::wasm2c::Wasm2cModsRuntime *)env);
    return runtime->apiObject.wr_sqlite3_bind_blob(WASMRawI64::make(stmt), WASMRawI32::make(a), WASMRawU64::make(in_blob_offset), WASMRawI32::make(n)).get();
}

u32 w2c_webrogue_wr_res_open(struct w2c_webrogue *env, u64 name, u32 nameLen) {
    auto runtime =
        ((webrogue::runtimes::wasm2c::Wasm2cModsRuntime *)env);
    return runtime->apiObject.wr_res_open(WASMRawU64::make(name), WASMRawU32::make(nameLen)).get();
}

u64 w2c_webrogue_wr_res_get_size(struct w2c_webrogue *env, u32 rd) {
    auto runtime =
        ((webrogue::runtimes::wasm2c::Wasm2cModsRuntime *)env);
    return runtime->apiObject.wr_res_get_size(WASMRawU32::make(rd)).get();
}

void w2c_webrogue_wr_res_get_data(struct w2c_webrogue *env, u32 rd, u64 outData) {
    auto runtime =
        ((webrogue::runtimes::wasm2c::Wasm2cModsRuntime *)env);
    return runtime->apiObject.wr_res_get_data(WASMRawU32::make(rd), WASMRawU64::make(outData));
}

void w2c_webrogue_wr_res_close(struct w2c_webrogue *env, u32 rd) {
    auto runtime =
        ((webrogue::runtimes::wasm2c::Wasm2cModsRuntime *)env);
    return runtime->apiObject.wr_res_close(WASMRawU32::make(rd));
}

}
