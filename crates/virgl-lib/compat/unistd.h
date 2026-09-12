#ifndef _COMPAT_UNISTD_H
#define _COMPAT_UNISTD_H

#include <stddef.h>
#include <sys/types.h>

#ifdef __cplusplus
extern "C" {
#endif

#ifndef STDIN_FILENO
#define STDIN_FILENO 0
#define STDOUT_FILENO 1
#define STDERR_FILENO 2
#endif

#ifndef SEEK_SET
#define SEEK_SET 0
#define SEEK_CUR 1
#define SEEK_END 2
#endif

#ifndef F_OK
#define F_OK 0
#define X_OK 1
#define W_OK 2
#define R_OK 4
#endif

#define _SC_PAGESIZE 30
#define _SC_PAGE_SIZE _SC_PAGESIZE
#define getpagesize() sysconf(_SC_PAGESIZE)

int close(int fd);
int read(int fd, void *buf, unsigned int count);
int write(int fd, const void *buf, unsigned int count);
long lseek(int fd, long offset, int whence);
int fsync(int fd);
int ftruncate(int fd, off_t length);
int dup(int fd);
int dup2(int fd, int fd2);
int pipe(int fds[2]);
int access(const char *path, int mode);
int unlink(const char *path);
int rmdir(const char *path);
int chdir(const char *path);
char *getcwd(char *buf, int size);
int getpid(void);
long sysconf(int name);
unsigned int sleep(unsigned int seconds);
int usleep(useconds_t usec);
int setenv(const char *name, const char *value, int overwrite);
int unsetenv(const char *name);

#ifdef __cplusplus
}
#endif

#endif
