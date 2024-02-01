#include "ModsRuntime.hpp"
#include "WASIObject.hpp"
#include <cassert>
#include <cstddef>
#include <cstdint>
#include <cstdlib>
#include <cstring>
#include <string>
#include <vector>

#include "uvwasi.h"
#include "wasi_templates.hpp"
#include "wasm_types.hpp"

namespace webrogue {
namespace core {

WASI_FUNCTION_IMPL(WASMRawU32, fd_close, (WASMRawU32 fd)) {
    return WASMRawU32::make(uvwasi_fd_close(uvwasi, fd.get()));
}

WASI_FUNCTION_IMPL(WASMRawU32, fd_fdstat_get,
                   (WASMRawU32 fd, WASMRawU32 out_offset)) {
    uint32_t const buf = out_offset.get();

    if (!runtime->isVMRangeValid(buf, 24))
        return E_BAD_ADDR;

    uvwasi_fdstat_t stat;
    uvwasi_errno_t const ret = uvwasi_fd_fdstat_get(uvwasi, fd.get(), &stat);

    if (ret != UVWASI_ESUCCESS) {
        return WASMRawU32::make(ret);
    }

    runtime->setVMDataZeros(buf, 24);
    runtime->setVMInt<uint8_t>(buf + 0, stat.fs_filetype);
    runtime->setVMInt<uint16_t>(buf + 2, stat.fs_flags);
    runtime->setVMInt<uint64_t>(buf + 8, stat.fs_rights_base);
    runtime->setVMInt<uint64_t>(buf + 16, stat.fs_rights_inheriting);
    return WASMRawU32::make(ret);
}

WASI_FUNCTION_IMPL(WASMRawI32, fd_fdstat_set_flags,
                   (WASMRawI32 a, WASMRawI32 b)) {
    abort();
    return WASMRawI32::make(0);
}

WASI_FUNCTION_IMPL(WASMRawU32, fd_prestat_dir_name,
                   (WASMRawU32 fd, WASMRawU32 path, WASMRawU32 path_len)) {

    if (!runtime->isVMRangeValid(path.get(), path_len.get()))
        return E_BAD_ADDR;

    std::vector<char> hostPath;
    hostPath.resize(path_len.get());
    runtime->getVMData(hostPath.data(), path.get(), path_len.get());

    uvwasi_errno_t const ret = uvwasi_fd_prestat_dir_name(
        uvwasi, fd.get(), hostPath.data(), hostPath.size());

    runtime->setVMData(hostPath.data(), path.get(), path_len.get());

    return WASMRawU32::make(ret);
}

WASI_FUNCTION_IMPL(WASMRawU32, fd_prestat_get,
                   (WASMRawU32 fd, WASMRawU32 buf)) {
    if (!runtime->isVMRangeValid(buf.get(), 8))
        return E_BAD_ADDR;

    uvwasi_prestat_t prestat;

    uvwasi_errno_t const ret =
        uvwasi_fd_prestat_get(uvwasi, fd.get(), &prestat);

    if (ret != UVWASI_ESUCCESS) {
        return WASMRawU32::make(ret);
    }

    runtime->setVMInt<uint32_t>(buf.get() + 0, prestat.pr_type);
    runtime->setVMInt<uint32_t>(buf.get() + 4, prestat.u.dir.pr_name_len);
    return WASMRawU32::make(ret);
}

WASI_FUNCTION_IMPL(WASMRawU32, fd_read,
                   (WASMRawU32 fd, WASMRawU32 wasi_iovs, WASMRawU32 iovs_len,
                    WASMRawU32 nread)) {
    if (!runtime->isVMRangeValid(wasi_iovs.get(),
                                 iovs_len.get() * sizeof(uvwasi_iovec_t)))
        return E_BAD_ADDR;

    if (!runtime->isVMRangeValid(nread.get(), sizeof(uvwasi_size_t)))
        return E_BAD_ADDR;

    if (iovs_len.get() > 32)
        return WASMRawU32::make(UVWASI_EINVAL);
    uvwasi_iovec_t iovs[32];
    std::vector<char> buffers[32];
    uint32_t bufferOffsets[32];
    uint32_t bufferLen[32];

    uvwasi_size_t numRead;
    uvwasi_errno_t ret;

    for (uvwasi_size_t i = 0; i < iovs_len.get(); ++i) {
        uint32_t const buf = runtime->getVMInt<uint32_t>(
            wasi_iovs.get() + sizeof(uint32_t) * (i * 2));
        uint32_t const bufLen = runtime->getVMInt<uint32_t>(
            wasi_iovs.get() + sizeof(uint32_t) * (i * 2 + 1));
        if (!runtime->isVMRangeValid(buf, bufLen))
            return E_BAD_ADDR;
        bufferOffsets[i] = buf;
        bufferLen[i] = bufLen;
        buffers[i].resize(bufLen);
        iovs[i].buf = buffers[i].data();
        iovs[i].buf_len = bufLen;
        runtime->getVMData(iovs[i].buf, bufferOffsets[i], bufferLen[i]);
    }

    ret = uvwasi_fd_read(uvwasi, fd.get(), iovs, iovs_len.get(), &numRead);
    for (uvwasi_size_t i = 0; i < iovs_len.get(); ++i) {
        runtime->setVMData(iovs[i].buf, bufferOffsets[i], bufferLen[i]);
    }

    runtime->setVMInt<uint32_t>(nread.get(), numRead);
    return WASMRawU32::make(ret);
}

WASI_FUNCTION_IMPL(WASMRawU32, fd_readdir,
                   (WASMRawU32 a, WASMRawU32 b, WASMRawU32 c, WASMRawU64 d,
                    WASMRawU32 e)) {
    abort();
    return WASMRawU32::make(8);
}

WASI_FUNCTION_IMPL(WASMRawU32, fd_seek,
                   (WASMRawI32 fd, WASMRawI64 offset, WASMRawI32 wasi_whence,
                    WASMRawU32 result)) {
    if (!runtime->isVMRangeValid(result.get(), sizeof(uvwasi_filesize_t)))
        return E_BAD_ADDR;

    uvwasi_whence_t whence = -1;
    switch (wasi_whence.get()) {
    case 0:
        whence = UVWASI_WHENCE_CUR;
        break;
    case 1:
        whence = UVWASI_WHENCE_END;
        break;
    case 2:
        whence = UVWASI_WHENCE_SET;
        break;
    }

    uvwasi_filesize_t pos;
    uvwasi_errno_t const ret =
        uvwasi_fd_seek(uvwasi, fd.get(), offset.get(), whence, &pos);

    runtime->setVMInt<uint64_t>(result.get(), pos);

    return WASMRawU32::make(ret);
}

WASI_FUNCTION_IMPL(WASMRawU32, fd_write,
                   (WASMRawU32 fd, WASMRawU32 wasi_iovs, WASMRawU32 iovs_len,
                    WASMRawU32 nwritten)) {
    if (!runtime->isVMRangeValid(wasi_iovs.get(),
                                 iovs_len.get() * sizeof(uvwasi_iovec_t)))
        return E_BAD_ADDR;

    if (!runtime->isVMRangeValid(nwritten.get(), sizeof(uvwasi_size_t)))
        return E_BAD_ADDR;

    if (iovs_len.get() > 32)
        return WASMRawU32::make(UVWASI_EINVAL);
    uvwasi_ciovec_t iovs[32];
    std::vector<char> buffers[32];
    uint32_t bufferOffsets[32];
    uint32_t bufferLen[32];

    uvwasi_size_t numWritten;
    uvwasi_errno_t ret;

    for (uvwasi_size_t i = 0; i < iovs_len.get(); ++i) {
        uint32_t const buf = runtime->getVMInt<uint32_t>(
            wasi_iovs.get() + sizeof(uint32_t) * (i * 2));
        uint32_t const bufLen = runtime->getVMInt<uint32_t>(
            wasi_iovs.get() + sizeof(uint32_t) * (i * 2 + 1));
        if (!runtime->isVMRangeValid(buf, bufLen))
            return E_BAD_ADDR;
        bufferOffsets[i] = buf;
        bufferLen[i] = bufLen;
        buffers[i].resize(bufLen);
        iovs[i].buf = buffers[i].data();
        iovs[i].buf_len = bufLen;
        runtime->getVMData(buffers[i].data(), bufferOffsets[i], bufferLen[i]);
    }

    ret = uvwasi_fd_write(uvwasi, fd.get(), iovs, iovs_len.get(), &numWritten);
    for (uvwasi_size_t i = 0; i < iovs_len.get(); ++i) {
        runtime->setVMData(iovs[i].buf, bufferOffsets[i], bufferLen[i]);
    }

    runtime->setVMInt<uint32_t>(nwritten.get(), numWritten);
    return WASMRawU32::make(ret);
}

WASI_FUNCTION_IMPL(WASMRawI32, path_filestat_get,
                   (WASMRawU32 a, WASMRawU32 b, WASMRawU32 c, WASMRawU32 d,
                    WASMRawU32 e)) {
    abort();
    return WASMRawI32::make(8);
}

WASI_FUNCTION_IMPL(WASMRawU32, path_open,
                   (WASMRawU32 dirfd, WASMRawU32 dirflags, WASMRawU32 path,
                    WASMRawU32 path_len, WASMRawU32 oflags,
                    WASMRawU64 fs_rights_base, WASMRawU64 fs_rights_inheriting,
                    WASMRawU32 fs_flags, WASMRawU32 fd)) {
    if (!runtime->isVMRangeValid(path.get(), path_len.get()))
        return E_BAD_ADDR;
    if (!runtime->isVMRangeValid(fd.get(), sizeof(uvwasi_fd_t)))
        return E_BAD_ADDR;

    uvwasi_fd_t uvfd;

    std::vector<char> pathData;
    pathData.reserve(path_len.get());
    runtime->getVMData(pathData.data(), path.get(), path_len.get());

    uvwasi_errno_t const ret =
        uvwasi_path_open(uvwasi, dirfd.get(), dirflags.get(), pathData.data(),
                         path_len.get(), oflags.get(), fs_rights_base.get(),
                         fs_rights_inheriting.get(), fs_flags.get(), &uvfd);

    runtime->setVMInt<uint32_t>(fd.get(), uvfd);
    return WASMRawU32::make(ret);
}

WASI_FUNCTION_IMPL(WASMRawU32, path_remove_directory,
                   (WASMRawU32 fd, WASMRawU32 path, WASMRawU32 path_len)) {
    if (!runtime->isVMRangeValid(path.get(), path_len.get()))
        return E_BAD_ADDR;

    std::vector<char> pathData;
    pathData.reserve(path_len.get());
    runtime->getVMData(pathData.data(), path.get(), path_len.get());

    uvwasi_errno_t const ret = uvwasi_path_remove_directory(
        uvwasi, fd.get(), pathData.data(), path_len.get());

    return WASMRawU32::make(ret);
}
WASI_FUNCTION_IMPL(WASMRawI32, path_rename,
                   (WASMRawU32 a, WASMRawU32 b, WASMRawU32 c, WASMRawU32 d,
                    WASMRawU32 e, WASMRawU32 f)) {
    abort();
    return WASMRawI32::make(8);
}
WASI_FUNCTION_IMPL(WASMRawU32, path_unlink_file,
                   (WASMRawU32 fd, WASMRawU32 path, WASMRawU32 path_len)) {
    if (!runtime->isVMRangeValid(path.get(), path_len.get()))
        return E_BAD_ADDR;

    std::vector<char> pathData;
    pathData.reserve(path_len.get());
    runtime->getVMData(pathData.data(), path.get(), path_len.get());

    uvwasi_errno_t const ret = uvwasi_path_unlink_file(
        uvwasi, fd.get(), pathData.data(), path_len.get());

    return WASMRawU32::make(ret);
}

} // namespace core
} // namespace webrogue
