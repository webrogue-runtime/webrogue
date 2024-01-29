#include "ModsRuntime.hpp"
#include "WASIObject.hpp"
#include "byteswap.hpp"
#include <cassert>
#include <cstddef>
#include <cstdint>
#include <cstdlib>
#include <cstring>
#include <string>
#include <vector>

#include "VFS.hpp"
#include "wasi_templates.hpp"
#include "wasm_types.hpp"

namespace webrogue {
namespace core {

WASI_FUNCTION_IMPL(WASMRawU32, fd_close, (WASMRawU32 fd)) {
    return WASMRawU32::make(vfs.close(fd.get()) ? 0 : 8);
}

WASI_FUNCTION_IMPL(WASMRawI32, fd_fdstat_get,
                   (WASMRawU32 fd, WASMRawU32 out_offset)) {
    nr_wasi_fdstat_t fdstat;
    fdstat.fs_filetype = byteswap<nr_wasi_filetype_t>(3); // 4 for file
    fdstat.fs_flags = byteswap<nr_wasi_fdflags_t>(0);
    fdstat.fs_rights_base = (uint64_t)-1;       // all rights
    fdstat.fs_rights_inheriting = (uint64_t)-1; // all rights
    WASI_CHECK(runtime->setVMData(&fdstat, out_offset.get(),
                                  sizeof(nr_wasi_fdstat_t)));
    return WASMRawI32::make(0);
}

WASI_FUNCTION_IMPL(WASMRawI32, fd_fdstat_set_flags,
                   (WASMRawI32 a, WASMRawI32 b)) {
    // TODO
    return WASMRawI32::make(0);
}

WASI_FUNCTION_IMPL(WASMRawI32, fd_prestat_dir_name,
                   (WASMRawU32 fd, WASMRawU32 out, WASMRawU32 len)) {
    std::string name;
    if (!vfs.preopendDirName(fd.get(), name)) {
        return WASMRawI32::make(8);
    }
    if (len.get() != name.size() + 1) {
        return WASMRawI32::make(8);
    }
    WASI_CHECK(runtime->setVMData(name.c_str(), out.get(), len.get()));
    return WASMRawI32::make(0);
}

WASI_FUNCTION_IMPL(WASMRawI32, fd_prestat_get,
                   (WASMRawU32 fd, WASMRawU32 out)) {
    std::string name;
    if (!vfs.preopendDirName(fd.get(), name)) {
        return WASMRawI32::make(8);
    }
    WASMI32 ret[2];
    ret[0] = WASMI32::make(0);
    ret[1] = WASMI32::make(name.size() + 1);
    WASI_CHECK(runtime->setVMData(ret, out.get(), sizeof(WASMI32) * 2));
    return WASMRawI32::make(0);
}

WASI_FUNCTION_IMPL(WASMRawI32, fd_read,
                   (WASMRawI32 fd, WASMRawI32 raw_wasi_iovs_offset,
                    WASMRawI32 iovs_len, WASMRawI32 out_nread_offset)) {

    size_t hostNread = 0;
    bool hasError = false;
    std::vector<uint8_t> hostData;
    for (int i = 0; i < iovs_len.get(); i++) {
        nr_wasi_iovec_t io;
        WASI_CHECK(runtime->getVMData(
            &io, alignedOffset<nr_wasi_iovec_t>(raw_wasi_iovs_offset.get(), i),
            sizeof(nr_wasi_iovec_t)));

        nr_wasi_size_t buffOffset = byteswap<nr_wasi_size_t>(io.buf);
        nr_wasi_size_t size = byteswap<nr_wasi_size_t>(io.buf_len);

        if (!size)
            continue;
        hostData.resize(size);

        size_t currentNread;

        if (!vfs.read(fd.get(), hostData.data(), size, currentNread)) {
            hasError = true;
            assert(false);
            break;
        }
        WASI_CHECK(runtime->setVMData(hostData.data(), buffOffset, size));
        hostNread += currentNread;
    }

    nr_wasi_size_t nread = byteswap<nr_wasi_size_t>(hostNread);

    WASI_CHECK(runtime->setVMData(&nread, out_nread_offset.get(),
                                  sizeof(nr_wasi_size_t)));
    return WASMRawI32::make(hasError ? 8 : 0);
}

WASI_FUNCTION_IMPL(WASMRawU32, fd_readdir,
                   (WASMRawU32 a, WASMRawU32 b, WASMRawU32 c, WASMRawU64 d,
                    WASMRawU32 e)) {
    assert(false);
    return WASMRawU32::make(8);
}

WASI_FUNCTION_IMPL(WASMRawI32, fd_seek,
                   (WASMRawI32 fd, WASMRawI64 offset, WASMRawI32 whence,
                    WASMRawU32 out_pos_offset)) {
    size_t hostPos;
    if (vfs.seek(fd.get(), offset.get(), whence.get(), hostPos)) {
        nr_wasi_filesize_t pos = byteswap<nr_wasi_filesize_t>(hostPos);
        WASI_CHECK(runtime->setVMData(&pos, out_pos_offset.get(),
                                      sizeof(nr_wasi_filesize_t)));
        return WASMRawI32::make(0);
    }
    return WASMRawI32::make(8);
}

WASI_FUNCTION_IMPL(WASMRawI32, fd_write,
                   (WASMRawU32 fd, WASMRawU32 raw_wasi_iovs_offset,
                    WASMRawU32 iovs_len, WASMRawU32 out_nwritten_offset)) {

    size_t hostNwritten = 0;
    bool hasError = false;
    std::vector<uint8_t> hostData;
    for (int i = 0; i < iovs_len.get(); i++) {
        nr_wasi_iovec_t io;
        WASI_CHECK(runtime->getVMData(
            &io, alignedOffset<nr_wasi_iovec_t>(raw_wasi_iovs_offset.get(), i),
            sizeof(nr_wasi_iovec_t)));

        nr_wasi_size_t buffOffset = byteswap<nr_wasi_size_t>(io.buf);
        nr_wasi_size_t size = byteswap<nr_wasi_size_t>(io.buf_len);

        if (!size)
            continue;
        hostData.resize(size);
        WASI_CHECK(runtime->getVMData(hostData.data(), buffOffset, size));

        if (!vfs.write(fd.get(), hostData.data(), size)) {
            hasError = true;
            assert(false);
            break;
        }
        hostNwritten += size;
    }

    nr_wasi_size_t nwritten = byteswap<nr_wasi_size_t>(hostNwritten);

    WASI_CHECK(runtime->setVMData(&nwritten, out_nwritten_offset.get(),
                                  sizeof(nr_wasi_size_t)));
    return WASMRawI32::make(hasError ? 8 : 0);
}

WASI_FUNCTION_IMPL(WASMRawI32, path_filestat_get,
                   (WASMRawU32 a, WASMRawU32 b, WASMRawU32 c, WASMRawU32 d,
                    WASMRawU32 e)) {
    assert(false);
    return WASMRawI32::make(8);
}

WASI_FUNCTION_IMPL(WASMRawI32, path_open,
                   (WASMRawU32 dirfd, WASMRawU32 dirflags,
                    WASMRawU32 in_path_offset, WASMRawU32 path_len,
                    WASMRawU32 oflags, WASMRawU64 fs_rights_base,
                    WASMRawU64 fs_rights_inheriting, WASMRawU32 fs_flags,
                    WASMRawU32 out_fd_offset)) {

    std::vector<char> pathData;
    pathData.resize(path_len.get() + 1);
    WASI_CHECK(runtime->getVMData(pathData.data(), in_path_offset.get(),
                                  path_len.get()));
    pathData[path_len.get()] = '\0';

    std::string pathString = pathData.data();
    if (pathString.size() != path_len.get()) {
        assert(false);
        return WASMRawI32::make(28);
    }
    size_t outFd;
    if (!vfs.open(pathString, outFd, fs_flags.get() && __WASI_FDFLAGS_APPEND)) {
        assert(false);
        return WASMRawI32::make(8);
    }

    nr_wasi_fd_t fd = byteswap<nr_wasi_fd_t>(outFd);

    WASI_CHECK(
        runtime->setVMData(&fd, out_fd_offset.get(), sizeof(nr_wasi_fd_t)))

    return WASMRawI32::make(0);
}

WASI_FUNCTION_IMPL(WASMRawI32, path_remove_directory,
                   (WASMRawU32 a, WASMRawU32 b, WASMRawU32 c)) {
    assert(false);
    return WASMRawI32::make(8);
}
WASI_FUNCTION_IMPL(WASMRawI32, path_rename,
                   (WASMRawU32 a, WASMRawU32 b, WASMRawU32 c, WASMRawU32 d,
                    WASMRawU32 e, WASMRawU32 f)) {
    assert(false);
    return WASMRawI32::make(8);
}
WASI_FUNCTION_IMPL(WASMRawI32, path_unlink_file,
                   (WASMRawU32 a, WASMRawU32 b, WASMRawU32 c)) {
    assert(false);
    return WASMRawI32::make(8);
}

} // namespace core
} // namespace webrogue
