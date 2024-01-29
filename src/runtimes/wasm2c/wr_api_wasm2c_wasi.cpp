#include "wasm2c_runtime.hpp"
using namespace webrogue::core;

// clang-format off
extern "C" {
u32 w2c_wasi__snapshot__preview1_environ_get(struct w2c_wasi__snapshot__preview1 *env, u32 ptrs, u32 buff) {
    auto runtime =
        ((webrogue::runtimes::wasm2c::Wasm2cModsRuntime *)env);
    return runtime->wasiObject.environ_get(WASMRawU32::make(ptrs), WASMRawU32::make(buff)).get();
}

u32 w2c_wasi__snapshot__preview1_environ_sizes_get(struct w2c_wasi__snapshot__preview1 *env, u32 count_offset, u32 buffsize_offset) {
    auto runtime =
        ((webrogue::runtimes::wasm2c::Wasm2cModsRuntime *)env);
    return runtime->wasiObject.environ_sizes_get(WASMRawI32::make(count_offset), WASMRawI32::make(buffsize_offset)).get();
}

u32 w2c_wasi__snapshot__preview1_clock_time_get(struct w2c_wasi__snapshot__preview1 *env, u32 clk_id, u64 precision, u32 out_time_offset) {
    auto runtime =
        ((webrogue::runtimes::wasm2c::Wasm2cModsRuntime *)env);
    return runtime->wasiObject.clock_time_get(WASMRawU32::make(clk_id), WASMRawU64::make(precision), WASMRawU32::make(out_time_offset)).get();
}

u32 w2c_wasi__snapshot__preview1_fd_close(struct w2c_wasi__snapshot__preview1 *env, u32 fd) {
    auto runtime =
        ((webrogue::runtimes::wasm2c::Wasm2cModsRuntime *)env);
    return runtime->wasiObject.fd_close(WASMRawU32::make(fd)).get();
}

u32 w2c_wasi__snapshot__preview1_fd_fdstat_get(struct w2c_wasi__snapshot__preview1 *env, u32 fd, u32 out_offset) {
    auto runtime =
        ((webrogue::runtimes::wasm2c::Wasm2cModsRuntime *)env);
    return runtime->wasiObject.fd_fdstat_get(WASMRawU32::make(fd), WASMRawU32::make(out_offset)).get();
}

u32 w2c_wasi__snapshot__preview1_fd_fdstat_set_flags(struct w2c_wasi__snapshot__preview1 *env, u32 a, u32 b) {
    auto runtime =
        ((webrogue::runtimes::wasm2c::Wasm2cModsRuntime *)env);
    return runtime->wasiObject.fd_fdstat_set_flags(WASMRawI32::make(a), WASMRawI32::make(b)).get();
}

u32 w2c_wasi__snapshot__preview1_fd_prestat_dir_name(struct w2c_wasi__snapshot__preview1 *env, u32 fd, u32 out, u32 len) {
    auto runtime =
        ((webrogue::runtimes::wasm2c::Wasm2cModsRuntime *)env);
    return runtime->wasiObject.fd_prestat_dir_name(WASMRawU32::make(fd), WASMRawU32::make(out), WASMRawU32::make(len)).get();
}

u32 w2c_wasi__snapshot__preview1_fd_prestat_get(struct w2c_wasi__snapshot__preview1 *env, u32 fd, u32 out) {
    auto runtime =
        ((webrogue::runtimes::wasm2c::Wasm2cModsRuntime *)env);
    return runtime->wasiObject.fd_prestat_get(WASMRawU32::make(fd), WASMRawU32::make(out)).get();
}

u32 w2c_wasi__snapshot__preview1_fd_read(struct w2c_wasi__snapshot__preview1 *env, u32 a, u32 b, u32 c, u32 g) {
    auto runtime =
        ((webrogue::runtimes::wasm2c::Wasm2cModsRuntime *)env);
    return runtime->wasiObject.fd_read(WASMRawI32::make(a), WASMRawI32::make(b), WASMRawI32::make(c), WASMRawI32::make(g)).get();
}

u32 w2c_wasi__snapshot__preview1_fd_readdir(struct w2c_wasi__snapshot__preview1 *env, u32 a, u32 b, u32 c, u64 d, u32 e) {
    auto runtime =
        ((webrogue::runtimes::wasm2c::Wasm2cModsRuntime *)env);
    return runtime->wasiObject.fd_readdir(WASMRawU32::make(a), WASMRawU32::make(b), WASMRawU32::make(c), WASMRawU64::make(d), WASMRawU32::make(e)).get();
}

u32 w2c_wasi__snapshot__preview1_fd_seek(struct w2c_wasi__snapshot__preview1 *env, u32 fd, u64 offset, u32 whence, u32 out_pos_offset) {
    auto runtime =
        ((webrogue::runtimes::wasm2c::Wasm2cModsRuntime *)env);
    return runtime->wasiObject.fd_seek(WASMRawI32::make(fd), WASMRawI64::make(offset), WASMRawI32::make(whence), WASMRawU32::make(out_pos_offset)).get();
}

u32 w2c_wasi__snapshot__preview1_fd_write(struct w2c_wasi__snapshot__preview1 *env, u32 fd, u32 raw_wasi_iovs_offset, u32 iovs_len, u32 out_nwritten_offset) {
    auto runtime =
        ((webrogue::runtimes::wasm2c::Wasm2cModsRuntime *)env);
    return runtime->wasiObject.fd_write(WASMRawU32::make(fd), WASMRawU32::make(raw_wasi_iovs_offset), WASMRawU32::make(iovs_len), WASMRawU32::make(out_nwritten_offset)).get();
}

u32 w2c_wasi__snapshot__preview1_path_filestat_get(struct w2c_wasi__snapshot__preview1 *env, u32 a, u32 b, u32 c, u32 d, u32 e) {
    auto runtime =
        ((webrogue::runtimes::wasm2c::Wasm2cModsRuntime *)env);
    return runtime->wasiObject.path_filestat_get(WASMRawU32::make(a), WASMRawU32::make(b), WASMRawU32::make(c), WASMRawU32::make(d), WASMRawU32::make(e)).get();
}

u32 w2c_wasi__snapshot__preview1_path_open(struct w2c_wasi__snapshot__preview1 *env, u32 dirfd, u32 dirflags, u32 in_path_offset, u32 path_len, u32 oflags, u64 fs_rights_base, u64 fs_rights_inheriting, u32 fs_flags, u32 out_fd_offset) {
    auto runtime =
        ((webrogue::runtimes::wasm2c::Wasm2cModsRuntime *)env);
    return runtime->wasiObject.path_open(WASMRawU32::make(dirfd), WASMRawU32::make(dirflags), WASMRawU32::make(in_path_offset), WASMRawU32::make(path_len), WASMRawU32::make(oflags), WASMRawU64::make(fs_rights_base), WASMRawU64::make(fs_rights_inheriting), WASMRawU32::make(fs_flags), WASMRawU32::make(out_fd_offset)).get();
}

u32 w2c_wasi__snapshot__preview1_path_remove_directory(struct w2c_wasi__snapshot__preview1 *env, u32 a, u32 b, u32 c) {
    auto runtime =
        ((webrogue::runtimes::wasm2c::Wasm2cModsRuntime *)env);
    return runtime->wasiObject.path_remove_directory(WASMRawU32::make(a), WASMRawU32::make(b), WASMRawU32::make(c)).get();
}

u32 w2c_wasi__snapshot__preview1_path_rename(struct w2c_wasi__snapshot__preview1 *env, u32 a, u32 b, u32 c, u32 d, u32 e, u32 f) {
    auto runtime =
        ((webrogue::runtimes::wasm2c::Wasm2cModsRuntime *)env);
    return runtime->wasiObject.path_rename(WASMRawU32::make(a), WASMRawU32::make(b), WASMRawU32::make(c), WASMRawU32::make(d), WASMRawU32::make(e), WASMRawU32::make(f)).get();
}

u32 w2c_wasi__snapshot__preview1_path_unlink_file(struct w2c_wasi__snapshot__preview1 *env, u32 a, u32 b, u32 c) {
    auto runtime =
        ((webrogue::runtimes::wasm2c::Wasm2cModsRuntime *)env);
    return runtime->wasiObject.path_unlink_file(WASMRawU32::make(a), WASMRawU32::make(b), WASMRawU32::make(c)).get();
}

u32 w2c_wasi__snapshot__preview1_random_get(struct w2c_wasi__snapshot__preview1 *env, u32 a, u32 b) {
    auto runtime =
        ((webrogue::runtimes::wasm2c::Wasm2cModsRuntime *)env);
    return runtime->wasiObject.random_get(WASMRawU32::make(a), WASMRawU32::make(b)).get();
}

u32 w2c_wasi__snapshot__preview1_fd_sync(struct w2c_wasi__snapshot__preview1 *env, u32 a) {
    auto runtime =
        ((webrogue::runtimes::wasm2c::Wasm2cModsRuntime *)env);
    return runtime->wasiObject.fd_sync(WASMRawU32::make(a)).get();
}

u32 w2c_wasi__snapshot__preview1_fd_allocate(struct w2c_wasi__snapshot__preview1 *env, u32 a, u64 b, u64 c) {
    auto runtime =
        ((webrogue::runtimes::wasm2c::Wasm2cModsRuntime *)env);
    return runtime->wasiObject.fd_allocate(WASMRawU32::make(a), WASMRawU64::make(b), WASMRawU64::make(c)).get();
}

u32 w2c_wasi__snapshot__preview1_fd_datasync(struct w2c_wasi__snapshot__preview1 *env, u32 a) {
    auto runtime =
        ((webrogue::runtimes::wasm2c::Wasm2cModsRuntime *)env);
    return runtime->wasiObject.fd_datasync(WASMRawU32::make(a)).get();
}

u32 w2c_wasi__snapshot__preview1_fd_tell(struct w2c_wasi__snapshot__preview1 *env, u32 a, u32 b) {
    auto runtime =
        ((webrogue::runtimes::wasm2c::Wasm2cModsRuntime *)env);
    return runtime->wasiObject.fd_tell(WASMRawU32::make(a), WASMRawU32::make(b)).get();
}

u32 w2c_wasi__snapshot__preview1_fd_pwrite(struct w2c_wasi__snapshot__preview1 *env, u32 a, u32 b, u32 c, u64 d, u32 e) {
    auto runtime =
        ((webrogue::runtimes::wasm2c::Wasm2cModsRuntime *)env);
    return runtime->wasiObject.fd_pwrite(WASMRawU32::make(a), WASMRawU32::make(b), WASMRawU32::make(c), WASMRawU64::make(d), WASMRawU32::make(e)).get();
}

u32 w2c_wasi__snapshot__preview1_clock_res_get(struct w2c_wasi__snapshot__preview1 *env, u32 a, u32 b) {
    auto runtime =
        ((webrogue::runtimes::wasm2c::Wasm2cModsRuntime *)env);
    return runtime->wasiObject.clock_res_get(WASMRawU32::make(a), WASMRawU32::make(b)).get();
}

u32 w2c_wasi__snapshot__preview1_fd_renumber(struct w2c_wasi__snapshot__preview1 *env, u32 a, u32 b) {
    auto runtime =
        ((webrogue::runtimes::wasm2c::Wasm2cModsRuntime *)env);
    return runtime->wasiObject.fd_renumber(WASMRawU32::make(a), WASMRawU32::make(b)).get();
}

u32 w2c_wasi__snapshot__preview1_fd_pread(struct w2c_wasi__snapshot__preview1 *env, u32 a, u32 b, u32 c, u64 d, u32 e) {
    auto runtime =
        ((webrogue::runtimes::wasm2c::Wasm2cModsRuntime *)env);
    return runtime->wasiObject.fd_pread(WASMRawU32::make(a), WASMRawU32::make(b), WASMRawU32::make(c), WASMRawU64::make(d), WASMRawU32::make(e)).get();
}

u32 w2c_wasi__snapshot__preview1_path_create_directory(struct w2c_wasi__snapshot__preview1 *env, u32 a, u32 b, u32 c) {
    auto runtime =
        ((webrogue::runtimes::wasm2c::Wasm2cModsRuntime *)env);
    return runtime->wasiObject.path_create_directory(WASMRawU32::make(a), WASMRawU32::make(b), WASMRawU32::make(c)).get();
}

u32 w2c_wasi__snapshot__preview1_sched_yield(struct w2c_wasi__snapshot__preview1 *env) {
    auto runtime =
        ((webrogue::runtimes::wasm2c::Wasm2cModsRuntime *)env);
    return runtime->wasiObject.sched_yield().get();
}

u32 w2c_wasi__snapshot__preview1_fd_filestat_set_times(struct w2c_wasi__snapshot__preview1 *env, u32 a, u64 b, u64 c, u32 d) {
    auto runtime =
        ((webrogue::runtimes::wasm2c::Wasm2cModsRuntime *)env);
    return runtime->wasiObject.fd_filestat_set_times(WASMRawU32::make(a), WASMRawU64::make(b), WASMRawU64::make(c), WASMRawU32::make(d)).get();
}

u32 w2c_wasi__snapshot__preview1_args_sizes_get(struct w2c_wasi__snapshot__preview1 *env, u32 a, u32 b) {
    auto runtime =
        ((webrogue::runtimes::wasm2c::Wasm2cModsRuntime *)env);
    return runtime->wasiObject.args_sizes_get(WASMRawU32::make(a), WASMRawU32::make(b)).get();
}

u32 w2c_wasi__snapshot__preview1_fd_filestat_set_size(struct w2c_wasi__snapshot__preview1 *env, u32 a, u64 b) {
    auto runtime =
        ((webrogue::runtimes::wasm2c::Wasm2cModsRuntime *)env);
    return runtime->wasiObject.fd_filestat_set_size(WASMRawU32::make(a), WASMRawU64::make(b)).get();
}

u32 w2c_wasi__snapshot__preview1_sock_recv(struct w2c_wasi__snapshot__preview1 *env, u32 a, u32 b, u32 c, u32 d, u32 e, u32 f) {
    auto runtime =
        ((webrogue::runtimes::wasm2c::Wasm2cModsRuntime *)env);
    return runtime->wasiObject.sock_recv(WASMRawU32::make(a), WASMRawU32::make(b), WASMRawU32::make(c), WASMRawU32::make(d), WASMRawU32::make(e), WASMRawU32::make(f)).get();
}

u32 w2c_wasi__snapshot__preview1_fd_filestat_get(struct w2c_wasi__snapshot__preview1 *env, u32 a, u32 b) {
    auto runtime =
        ((webrogue::runtimes::wasm2c::Wasm2cModsRuntime *)env);
    return runtime->wasiObject.fd_filestat_get(WASMRawU32::make(a), WASMRawU32::make(b)).get();
}

u32 w2c_wasi__snapshot__preview1_sock_shutdown(struct w2c_wasi__snapshot__preview1 *env, u32 a, u32 b) {
    auto runtime =
        ((webrogue::runtimes::wasm2c::Wasm2cModsRuntime *)env);
    return runtime->wasiObject.sock_shutdown(WASMRawU32::make(a), WASMRawU32::make(b)).get();
}

u32 w2c_wasi__snapshot__preview1_poll_oneoff(struct w2c_wasi__snapshot__preview1 *env, u32 a, u32 b, u32 c, u32 d) {
    auto runtime =
        ((webrogue::runtimes::wasm2c::Wasm2cModsRuntime *)env);
    return runtime->wasiObject.poll_oneoff(WASMRawU32::make(a), WASMRawU32::make(b), WASMRawU32::make(c), WASMRawU32::make(d)).get();
}

u32 w2c_wasi__snapshot__preview1_path_link(struct w2c_wasi__snapshot__preview1 *env, u32 a, u32 b, u32 c, u32 d, u32 e, u32 f, u32 g) {
    auto runtime =
        ((webrogue::runtimes::wasm2c::Wasm2cModsRuntime *)env);
    return runtime->wasiObject.path_link(WASMRawU32::make(a), WASMRawU32::make(b), WASMRawU32::make(c), WASMRawU32::make(d), WASMRawU32::make(e), WASMRawU32::make(f), WASMRawU32::make(g)).get();
}

u32 w2c_wasi__snapshot__preview1_sock_accept(struct w2c_wasi__snapshot__preview1 *env, u32 a, u32 b, u32 c) {
    auto runtime =
        ((webrogue::runtimes::wasm2c::Wasm2cModsRuntime *)env);
    return runtime->wasiObject.sock_accept(WASMRawU32::make(a), WASMRawU32::make(b), WASMRawU32::make(c)).get();
}

u32 w2c_wasi__snapshot__preview1_path_readlink(struct w2c_wasi__snapshot__preview1 *env, u32 a, u32 b, u32 c, u32 d, u32 e, u32 f) {
    auto runtime =
        ((webrogue::runtimes::wasm2c::Wasm2cModsRuntime *)env);
    return runtime->wasiObject.path_readlink(WASMRawU32::make(a), WASMRawU32::make(b), WASMRawU32::make(c), WASMRawU32::make(d), WASMRawU32::make(e), WASMRawU32::make(f)).get();
}

u32 w2c_wasi__snapshot__preview1_fd_advise(struct w2c_wasi__snapshot__preview1 *env, u32 a, u64 b, u64 c, u32 d) {
    auto runtime =
        ((webrogue::runtimes::wasm2c::Wasm2cModsRuntime *)env);
    return runtime->wasiObject.fd_advise(WASMRawU32::make(a), WASMRawU64::make(b), WASMRawU64::make(c), WASMRawU32::make(d)).get();
}

u32 w2c_wasi__snapshot__preview1_path_filestat_set_times(struct w2c_wasi__snapshot__preview1 *env, u32 a, u32 b, u32 c, u32 d, u64 e, u64 f, u32 g) {
    auto runtime =
        ((webrogue::runtimes::wasm2c::Wasm2cModsRuntime *)env);
    return runtime->wasiObject.path_filestat_set_times(WASMRawU32::make(a), WASMRawU32::make(b), WASMRawU32::make(c), WASMRawU32::make(d), WASMRawU64::make(e), WASMRawU64::make(f), WASMRawU32::make(g)).get();
}

u32 w2c_wasi__snapshot__preview1_args_get(struct w2c_wasi__snapshot__preview1 *env, u32 a, u32 b) {
    auto runtime =
        ((webrogue::runtimes::wasm2c::Wasm2cModsRuntime *)env);
    return runtime->wasiObject.args_get(WASMRawU32::make(a), WASMRawU32::make(b)).get();
}

u32 w2c_wasi__snapshot__preview1_path_symlink(struct w2c_wasi__snapshot__preview1 *env, u32 a, u32 b, u32 c, u32 d, u32 e) {
    auto runtime =
        ((webrogue::runtimes::wasm2c::Wasm2cModsRuntime *)env);
    return runtime->wasiObject.path_symlink(WASMRawU32::make(a), WASMRawU32::make(b), WASMRawU32::make(c), WASMRawU32::make(d), WASMRawU32::make(e)).get();
}

u32 w2c_wasi__snapshot__preview1_sock_send(struct w2c_wasi__snapshot__preview1 *env, u32 a, u32 b, u32 c, u32 d, u32 e) {
    auto runtime =
        ((webrogue::runtimes::wasm2c::Wasm2cModsRuntime *)env);
    return runtime->wasiObject.sock_send(WASMRawU32::make(a), WASMRawU32::make(b), WASMRawU32::make(c), WASMRawU32::make(d), WASMRawU32::make(e)).get();
}

u32 w2c_wasi__snapshot__preview1_fd_fdstat_set_rights(struct w2c_wasi__snapshot__preview1 *env, u32 a, u64 b, u64 c) {
    auto runtime =
        ((webrogue::runtimes::wasm2c::Wasm2cModsRuntime *)env);
    return runtime->wasiObject.fd_fdstat_set_rights(WASMRawU32::make(a), WASMRawU64::make(b), WASMRawU64::make(c)).get();
}

}
