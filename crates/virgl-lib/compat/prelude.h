#ifndef _COMPAT_PRELUDE_H
#define _COMPAT_PRELUDE_H

/* This header is force-included (via /FI) into every translation unit of the
 * Windows build, mirroring what <unistd.h>/<time.h> would provide on POSIX.
 * Keep it tiny and side-effect free. */

#ifdef _WIN32

#ifndef _CRT_SECURE_NO_WARNINGS
#define _CRT_SECURE_NO_WARNINGS 1
#endif

#include <stdint.h>
#include <time.h>

#if !defined(CLOCK_REALTIME)
#define CLOCK_REALTIME 0
#define CLOCK_MONOTONIC 1
#define CLOCK_PROCESS_CPUTIME_ID 2
#define CLOCK_THREAD_CPUTIME_ID 3

int clock_gettime(int clock_id, struct timespec *tp);
#endif

#if !defined(CLOCK_NANOSLEEP_DECLARED)
#define CLOCK_NANOSLEEP_DECLARED
#define TIMER_ABSTIME 1

int clock_nanosleep(int clock_id, int flags, const struct timespec *req,
                    struct timespec *rem);
#endif

#endif /* _WIN32 */

#endif
