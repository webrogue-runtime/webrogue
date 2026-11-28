#ifndef _COMPAT_SYS_IOCTL_H
#define _COMPAT_SYS_IOCTL_H

#include <stdarg.h>
#include <sys/ioccom.h>

#ifdef __cplusplus
extern "C" {
#endif

int ioctl(int fd, unsigned long request, ...);

#ifdef __cplusplus
}
#endif

#endif
