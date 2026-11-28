#ifndef _COMPAT_SYS_RESOURCE_H
#define _COMPAT_SYS_RESOURCE_H

#include <sys/types.h>

#ifdef __cplusplus
extern "C" {
#endif

/* struct timeval is defined unconditionally by <winsock.h> / <winsock2.h>
 * whenever Vulkan headers (and thereby <windows.h>) are in the include chain,
 * so we must not define it here.  struct rusage mirrors it with plain longs. */
#define PRIO_PROCESS 0
#define PRIO_PGRP 1
#define PRIO_USER 2

#define RUSAGE_SELF 0
#define RUSAGE_CHILDREN (-1)

struct rlimit {
   long rlim_cur;
   long rlim_max;
};

struct rusage {
   long ru_utime_sec;
   long ru_utime_usec;
   long ru_stime_sec;
   long ru_stime_usec;
   long ru_maxrss;
   long ru_ixrss;
   long ru_idrss;
   long ru_isrss;
   long ru_minflt;
   long ru_majflt;
   long ru_nswap;
   long ru_inblock;
   long ru_oublock;
   long ru_msgsnd;
   long ru_msgrcv;
   long ru_nsignals;
   long ru_nvcsw;
   long ru_nivcsw;
};

int getpriority(int which, id_t who);
int setpriority(int which, id_t who, int prio);
int getrlimit(int resource, struct rlimit *rlim);
int setrlimit(int resource, const struct rlimit *rlim);
int getrusage(int who, struct rusage *usage);

#ifdef __cplusplus
}
#endif

#endif
