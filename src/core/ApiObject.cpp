#include "../../mods/core/include/common/colors.h"
#include "Config.hpp"
#include "ModsRuntime.hpp"
#include "byteswap.hpp"
#include "wasm_types.hpp"
#include <cassert>
#include <cstddef>
#include <cstdint>
#include <cstring>
#include <vector>

namespace webrogue {
namespace core {
ApiObject::ApiObject(ModsRuntime *pRuntime, Config *pConfig)
    : runtime(pRuntime), config(pConfig) {
}

#define WR_API_FUNCTION_IMPL(RET_TYPE, NAME, ARGS) RET_TYPE ApiObject::NAME ARGS

// rendering

WR_API_FUNCTION_IMPL(WASMRawI32, wr_get_render_width, ()) {
    if (!runtime->isInitialized) {
        assert(false);
        return WASMRawI32::make(-1);
    }
    return WASMRawI32::make(output->size().x);
}
WR_API_FUNCTION_IMPL(WASMRawI32, wr_get_render_height, ()) {
    if (!runtime->isInitialized) {
        assert(false);
        return WASMRawI32::make(-1);
    }
    return WASMRawI32::make(output->size().y);
}
WR_API_FUNCTION_IMPL(void, wr_start_color, ()) {
    if (!runtime->isInitialized) {
        assert(false);
        return;
    }
    output->startColor();
}
WR_API_FUNCTION_IMPL(WASMRawI32, wr_get_color_pairs_count, ()) {
    if (!runtime->isInitialized) {
        assert(false);
        return WASMRawI32::make(-1);
    }
    return WASMRawI32::make(output->getColorPairsCount());
}
WR_API_FUNCTION_IMPL(WASMRawI32, wr_get_colors_count, ()) {
    if (!runtime->isInitialized) {
        assert(false);
        return WASMRawI32::make(-1);
    }
    return WASMRawI32::make(output->getColorsCount());
}
WR_API_FUNCTION_IMPL(void, wr_set_color,
                     (WASMRawI32 color, WASMRawI32 r, WASMRawI32 g,
                      WASMRawI32 b)) {
    if (!runtime->isInitialized) {
        assert(false);
        return;
    }
    output->setColor(color.get(), r.get(), g.get(), b.get());
}
WR_API_FUNCTION_IMPL(void, wr_set_color_pair,
                     (WASMRawI32 color_pair, WASMRawI32 fg, WASMRawI32 bg)) {
    if (!runtime->isInitialized) {
        assert(false);
        return;
    }
    output->setColorPair(color_pair.get(), fg.get(), bg.get());
}
WR_API_FUNCTION_IMPL(void, wr_render_set_screen_data,
                     (WASMRawU64 in_buff_offset, WASMRawI64 size)) {

    if (size.get() != output->size().x * output->size().y) {
        assert(false);
        return;
    }
    if (!runtime->getVMData(output->getBuffer(), in_buff_offset.get(),
                            size.get() * sizeof(wr_glyph))) {
        assert(false);
        return;
    }
}
WR_API_FUNCTION_IMPL(void, wr_set_deadline, (WASMRawI32 ms)) {
    output->addDeadline(static_cast<float>(ms.get()) / 1000);
}
WR_API_FUNCTION_IMPL(WASMRawI32, wr_interrupt, ()) {
    if (!runtime->isInitialized) {
        assert(false);
        return WASMRawI32::make(-1);
    }
    webrogue_event consoleWriterExitEvent = {webrogue_event_type::None, 0, 0};
    output->endFrame();
    runtime->onFrameEnd();
    while (true) { // frames loop
        output->beginFrame();
        rawEvents.clear();
        bool gotDeadline = false;
        if (consoleWriterExitEvent.type != webrogue_event_type::None) {
            rawEvents.push_back({consoleWriterExitEvent.type,
                                 consoleWriterExitEvent.data1,
                                 consoleWriterExitEvent.data2, 0});
            consoleWriterExitEvent = {webrogue_event_type::None, 0, 0};
        }
        while (true) { // events loop
            webrogue_event event = output->getEvent();
            if (event.type == webrogue_event_type::None)
                break;
            if (event.type == webrogue_event_type::Deadline)
                gotDeadline = true;
            else
                rawEvents.push_back({event.type, event.data1, event.data2, 0});
            if (event.type == webrogue_event_type::Console)
                consoleWriter->isShown = true;
        } // events loop
        if (consoleWriter->isShown) {
            consoleWriterExitEvent = consoleWriter->present();
        } else {
            if (gotDeadline || !rawEvents.empty()) {
                return WASMRawI32::make(rawEvents.size());
            }
            output->lazyEnd();
        }
    } // frames loop
}

WR_API_FUNCTION_IMPL(void, wr_copy_events,
                     (WASMRawU64 out_buff_offset, WASMRawI64 size)) {
    if (!runtime->isInitialized) {
        assert(false);
        return;
    }
    if (size.get() != rawEvents.size()) {
        assert(false);
        return;
    }
    if (!runtime->setVMData(rawEvents.data(), out_buff_offset.get(),
                            size.get() * sizeof(webrogue_raw_event))) {
        assert(false);
        return;
    }
}

// debug

WR_API_FUNCTION_IMPL(void, wr_debug_print,
                     (WASMRawU64 in_buff_offset, WASMRawI64 size)) {
    std::vector<char> hostData;
    hostData.resize(size.get() + 1);
    if (!runtime->getVMData(hostData.data(), in_buff_offset.get(),
                            size.get())) {
        assert(false);
        return;
    }
    hostData[size.get()] = '\0';
    std::string hostString = hostData.data();
    *runtime->wrout << hostString;
}

// sqlite

WR_API_FUNCTION_IMPL(WASMRawI64, wr_sqlite3_errmsg_size, ()) {
    return WASMRawI64::make(strlen(sqlite3_errmsg(db->getDb())));
}
WR_API_FUNCTION_IMPL(void, wr_sqlite3_errmsg_get, (WASMRawU64 out_err_offset)) {
    const char *err = sqlite3_errmsg(db->getDb());
    size_t len = strlen(err);
    if (!runtime->setVMData(err, out_err_offset.get(), len + 1)) {
        assert(false);
        return;
    }
}
WR_API_FUNCTION_IMPL(WASMRawI32, wr_sqlite3_prepare_v2,
                     (WASMRawU64 in_zSql_offset, WASMRawI64 zSql_size,
                      WASMRawU64 out_ppStmt_offset,
                      WASMRawU64 out_pzTail_offset)) {
    std::vector<char> zSql;
    zSql.resize(zSql_size.get() + 1);

    if (!runtime->getVMData(zSql.data(), in_zSql_offset.get(),
                            zSql_size.get())) {
        assert(false);
        return WASMRawI32::make(-1);
    }
    zSql[zSql_size.get()] = '\0';

    const char *hostPzTail = nullptr;
    int64_t stmtId;

    int result = sqlite3_prepare_v2(
        db->getDb(), zSql.data(), zSql_size.get(), db->stmtNew(&stmtId),
        out_pzTail_offset.get() == 0 ? &hostPzTail : nullptr);
    if (out_pzTail_offset.get() != 0) {
        auto pzTailResult = in_zSql_offset.get() + (hostPzTail - zSql.data());
        switch (runtime->voidptrSize()) {
        case 4: {
            WASMRawU32 pztail = WASMRawU32::make(pzTailResult);
            if (!runtime->setVMData(&pztail, out_pzTail_offset.get(),
                                    sizeof(WASMRawU32))) {
                assert(false);
                return WASMRawI32::make(-1);
            }
            break;
        }

        case 8: {
            WASMRawU64 pztail = WASMRawU64::make(pzTailResult);
            if (!runtime->setVMData(&pztail, out_pzTail_offset.get(),
                                    sizeof(WASMRawU64))) {
                assert(false);
                return WASMRawI32::make(-1);
            }
            break;
        }
        }
    }
    if (result != SQLITE_OK)
        db->stmtDelete(stmtId);
    else {
        WASMRawI64 wasmStmtId = WASMRawI64::make(stmtId);
        if (!runtime->setVMData(&wasmStmtId, out_ppStmt_offset.get(),
                                sizeof(WASMRawI64))) {
            assert(false);
            return WASMRawI32::make(-1);
        }
    }
    return WASMRawI32::make(result);
}
WR_API_FUNCTION_IMPL(WASMRawI32, wr_sqlite3_step, (WASMRawI64 stmt)) {
    return WASMRawI32::make(sqlite3_step(db->stmtById(stmt.get())));
}
WR_API_FUNCTION_IMPL(WASMRawI64, wr_sqlite3_last_insert_rowid, ()) {
    return WASMRawI64::make(sqlite3_last_insert_rowid(db->getDb()));
}
WR_API_FUNCTION_IMPL(WASMRawI32, wr_sqlite3_changes, ()) {
    return WASMRawI32::make(sqlite3_changes(db->getDb()));
}
WR_API_FUNCTION_IMPL(WASMRawI32, wr_sqlite3_column_int,
                     (WASMRawI64 stmt, WASMRawI32 iCol)) {
    return WASMRawI32::make(
        sqlite3_column_int(db->stmtById(stmt.get()), iCol.get()));
}
WR_API_FUNCTION_IMPL(WASMRawI32, wr_sqlite3_column_type,
                     (WASMRawI64 stmt, WASMRawI32 iCol)) {
    return WASMRawI32::make(
        sqlite3_column_type(db->stmtById(stmt.get()), iCol.get()));
}
WR_API_FUNCTION_IMPL(WASMRawI64, wr_sqlite3_column_text_size,
                     (WASMRawI64 stmt, WASMRawI32 iCol)) {

    const char *text =
        (const char *)sqlite3_column_text(db->stmtById(stmt.get()), iCol.get());
    if (text)
        return WASMRawI64::make(strlen(text));
    return WASMRawI64::make(-1);
}
WR_API_FUNCTION_IMPL(void, wr_sqlite3_column_text_get,
                     (WASMRawI64 stmt, WASMRawI32 iCol,
                      WASMRawU64 out_result_offset)) {
    const char *text =
        (const char *)sqlite3_column_text(db->stmtById(stmt.get()), iCol.get());
    size_t len = strlen(text);
    if (!runtime->setVMData(text, out_result_offset.get(), len + 1)) {
        assert(false);
        return;
    }
}
WR_API_FUNCTION_IMPL(WASMRawF64, wr_sqlite3_column_double,
                     (WASMRawI64 stmt, WASMRawI32 iCol)) {
    return WASMRawF64::make(
        sqlite3_column_double(db->stmtById(stmt.get()), iCol.get()));
}
WR_API_FUNCTION_IMPL(WASMRawI64, wr_sqlite3_column_int64,
                     (WASMRawI64 stmt, WASMRawI32 iCol)) {
    return WASMRawI64::make(
        sqlite3_column_int64(db->stmtById(stmt.get()), iCol.get()));
}
WR_API_FUNCTION_IMPL(WASMRawI32, wr_sqlite3_column_bytes,
                     (WASMRawI64 stmt, WASMRawI32 iCol)) {
    return WASMRawI32::make(
        sqlite3_column_bytes(db->stmtById(stmt.get()), iCol.get()));
}
WR_API_FUNCTION_IMPL(void, wr_sqlite3_column_blob_get,
                     (WASMRawI64 stmt, WASMRawI32 iCol,
                      WASMRawU64 out_result_offset)) {
    size_t len = sqlite3_column_bytes(db->stmtById(stmt.get()), iCol.get());
    if (!runtime->setVMData(
            sqlite3_column_blob(db->stmtById(stmt.get()), iCol.get()),
            out_result_offset.get(), len)) {
        assert(false);
        return;
    }
}
WR_API_FUNCTION_IMPL(WASMRawI32, wr_sqlite3_finalize, (WASMRawI64 stmt)) {

    int result = sqlite3_finalize(db->stmtById(stmt.get()));
    if (result == SQLITE_OK)
        db->stmtDelete(stmt.get());
    return WASMRawI32::make(result);
}
WR_API_FUNCTION_IMPL(WASMRawI32, wr_sqlite3_reset, (WASMRawI64 stmt)) {
    return WASMRawI32::make(sqlite3_reset(db->stmtById(stmt.get())));
}
WR_API_FUNCTION_IMPL(WASMRawI32, wr_sqlite3_bind_int,
                     (WASMRawI64 stmt, WASMRawI32 a, WASMRawI32 b)) {

    return WASMRawI32::make(
        sqlite3_bind_int(db->stmtById(stmt.get()), a.get(), b.get()));
}
WR_API_FUNCTION_IMPL(WASMRawI32, wr_sqlite3_bind_null,
                     (WASMRawI64 stmt, WASMRawI32 a)) {

    return WASMRawI32::make(
        sqlite3_bind_null(db->stmtById(stmt.get()), a.get()));
}
WR_API_FUNCTION_IMPL(WASMRawI32, wr_sqlite3_bind_text,
                     (WASMRawI64 stmt, WASMRawI32 a, WASMRawU64 in_text_offset,
                      WASMRawI64 b_size)) {
    std::vector<char> hostText;
    hostText.resize(b_size.get() + 1);
    if (!runtime->getVMData(hostText.data(), in_text_offset.get(),
                            b_size.get())) {
        assert(false);
        return WASMRawI32::make(-1);
    }
    hostText[b_size.get()] = '\0';

    return WASMRawI32::make(sqlite3_bind_text(db->stmtById(stmt.get()), a.get(),
                                              hostText.data(), b_size.get(),
                                              SQLITE_TRANSIENT));
}
WR_API_FUNCTION_IMPL(WASMRawI32, wr_sqlite3_bind_double,
                     (WASMRawI64 stmt, WASMRawI32 a, WASMRawF64 b)) {
    return WASMRawI32::make(
        sqlite3_bind_double(db->stmtById(stmt.get()), a.get(), b.get()));
}
WR_API_FUNCTION_IMPL(WASMRawI32, wr_sqlite3_bind_int64,
                     (WASMRawI64 stmt, WASMRawI32 a, WASMRawI64 b)) {

    return WASMRawI32::make(
        sqlite3_bind_int64(db->stmtById(stmt.get()), a.get(), b.get()));
}
WR_API_FUNCTION_IMPL(WASMRawI32, wr_sqlite3_bind_blob,
                     (WASMRawI64 stmt, WASMRawI32 a, WASMRawU64 in_blob_offset,
                      WASMRawI32 n)) {
    std::vector<char> hostBlob;
    hostBlob.resize(n.get());
    if (!runtime->getVMData(hostBlob.data(), in_blob_offset.get(), n.get())) {
        assert(false);
        return WASMRawI32::make(-1);
    }
    return WASMRawI32::make(sqlite3_bind_blob(
        db->stmtById(stmt.get()), a.get(), hostBlob.data(), n.get(), nullptr));
}

} // namespace core
} // namespace webrogue
