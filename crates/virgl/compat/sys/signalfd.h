#ifndef _COMPAT_SYS_SIGNALFD_H
#define _COMPAT_SYS_SIGNALFD_H

#include <stdint.h>
#include <sys/types.h>

#ifdef __cplusplus
extern "C" {
#endif

/* Minimal sigset_t for the (omitted) Windows port; MSVC's <signal.h> does not
 * provide one. */
#ifndef _COMPAT_SIGSET_T
#define _COMPAT_SIGSET_T
typedef struct {
   uint64_t bits;
} sigset_t;
#endif

#define SFD_CLOEXEC 0
#define SFD_NONBLOCK 0

#define SIG_BLOCK 0
#define SIG_UNBLOCK 1
#define SIG_SETMASK 2

struct signalfd_siginfo {
   uint32_t ssi_signo;
   uint32_t ssi_errno;
   uint32_t ssi_code;
   uint32_t ssi_pid;
   uint32_t ssi_uid;
   int32_t ssi_fd;
   uint32_t ssi_tid;
   uint32_t ssi_band;
   uint32_t ssi_overrun;
   uint32_t ssi_trapno;
   int32_t ssi_status;
   int32_t ssi_int;
   uint64_t ssi_ptr;
   uint64_t ssi_utime;
   uint64_t ssi_stime;
   uint64_t ssi_addr;
   uint16_t ssi_addr_lsb;
   uint16_t __pad2;
   int32_t ssi_syscall;
   uint64_t ssi_call_addr;
   uint32_t ssi_arch;
   uint8_t __pad[28];
};

int signalfd(int fd, const sigset_t *mask, int flags);

#ifdef __cplusplus
}
#endif

#endif
