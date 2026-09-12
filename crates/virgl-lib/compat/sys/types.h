#ifndef _COMPAT_SYS_TYPES_H
#define _COMPAT_SYS_TYPES_H

#include <stddef.h>
#include <stdint.h>
#include <BaseTsd.h>

/* Mirror the types UCRT's <sys/types.h> / <sys/stat.h> rely on so that
 * system headers continue to work while our compat copy shadows the
 * Microsoft one. */
#ifndef _INO_T_DEFINED
#define _INO_T_DEFINED
typedef unsigned short _ino_t;
typedef _ino_t ino_t;
#endif

#ifndef _DEV_T_DEFINED
#define _DEV_T_DEFINED
typedef unsigned int _dev_t;
typedef _dev_t dev_t;
#endif

#ifndef _OFF_T_DEFINED
#define _OFF_T_DEFINED
typedef long _off_t;
typedef _off_t off_t;
#endif

#ifndef _OFF64_T_DEFINED
#define _OFF64_T_DEFINED
typedef __int64 _off64_t;
typedef _off64_t off64_t;
#endif

#ifndef _MODE_T_DEFINED
#define _MODE_T_DEFINED
typedef unsigned short _mode_t;
typedef _mode_t mode_t;
#endif

#ifndef _COMPAT_SSIZE_T
#define _COMPAT_SSIZE_T
typedef SSIZE_T ssize_t;
#endif

#ifndef _COMPAT_PID_T
#define _COMPAT_PID_T
typedef intptr_t pid_t;
#endif

#ifndef _COMPAT_USE
#define _COMPAT_USE
typedef unsigned int useconds_t;
#endif

#ifndef _COMPAT_ID_T
#define _COMPAT_ID_T
typedef int id_t;
typedef unsigned int idtype_t;
#endif

#ifndef _COMPAT_UID_GID_T
#define _COMPAT_UID_GID_T
typedef unsigned int uid_t;
typedef unsigned int gid_t;
#endif

#ifndef _COMPAT_NLINK_T
#define _COMPAT_NLINK_T
typedef unsigned short nlink_t;
#endif

#ifndef _COMPAT_BLKCNT
#define _COMPAT_BLKCNT
typedef long blkcnt_t;
typedef long blksize_t;
#endif

typedef unsigned char u_char;
typedef unsigned short u_short;
typedef unsigned int u_int;
typedef unsigned long u_long;

#endif
