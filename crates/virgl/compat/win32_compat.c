/*
 * Windows POSIX compatibility layer for the virglrenderer (webrogue) native
 * build.  unistd-style I/O, mmap, environment, getopt and a real in-process
 * SOCK_SEQPACKET socketpair transport (used by the same-process render server
 * for fd/message passing) are implemented on top of the Windows CRT and Win32
 * APIs.  Only genuinely separate-process POSIX machinery that Windows cannot
 * express (fork-based isolation, Unix-domain sockets with SCM_RIGHTS, dma-buf
 * fencing) falls back to abort() stubs.
 */

#ifndef WIN32_LEAN_AND_MEAN
#define WIN32_LEAN_AND_MEAN
#endif
#include <windows.h>

#include <errno.h>
#include <fcntl.h>
#include <io.h>
#include <direct.h>
#include <process.h>
#include <stddef.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include <sys/stat.h>

#include "sys/types.h"
#include "unistd.h"
#include "sys/mman.h"
#include "sys/uio.h"
#include "sys/socket.h"
#include "sys/ioctl.h"
#include "sys/resource.h"
#include "poll.h"
#include "sys/wait.h"
#include "sys/signalfd.h"
#include "getopt.h"
#include "dlfcn.h"

/* ------------------------------------------------------------------ */
/* In-process SOCK_SEQPACKET socketpair transport                      */
/*                                                                     */
/* The same-process render server (client, server and context workers  */
/* all live in one process) communicates over socketpair() endpoints   */
/* with sendmsg()/recvmsg() and SCM_RIGHTS fd passing.  Since every    */
/* endpoint shares this process, an fd is just a small integer and a   */
/* message + its fd list can be framed over an anonymous pipe.         */
/* ------------------------------------------------------------------ */

#define COMPAT_SOCK_BASE 0x4000
#define COMPAT_SOCK_MAX 128
#define COMPAT_SOCK_MAX_FD 8
#define COMPAT_SOCK_MAGIC 0x5647524cu /* "VGRL" */

/* Endpoints hold the pipe pair of one socket end.  Multiple fd slots may
 * reference the same endpoint (SCM_RIGHTS dup semantics): pipe handles are
 * released only when the last reference disappears.  Each endpoint also owns
 * a manual-reset event that is signaled when its peer endpoint is fully
 * closed, so poll() can report POLLHUP without needing pipe handles that may
 * have been closed by the other side. */
typedef struct compat_endpoint {
   HANDLE rx; /* read end: data coming into this endpoint */
   HANDLE tx; /* write end: data leaving this endpoint   */
   HANDLE event;
   CRITICAL_SECTION tx_lock;
   int type;
   int refs;        /* number of live fd slots referencing this endpoint */
   int peer;        /* endpoint index of the other side of the pair */
   int in_use;
} compat_endpoint;

static compat_endpoint compat_eps[COMPAT_SOCK_MAX];
static int compat_ep_fds[COMPAT_SOCK_MAX]; /* fd slot -> endpoint index or -1 */
static CRITICAL_SECTION compat_pool_lock;
static INIT_ONCE compat_pool_once = INIT_ONCE_STATIC_INIT;

static BOOL CALLBACK
compat_pool_init_once(PINIT_ONCE once, PVOID param, PVOID *ctx)
{
   (void)once;
   (void)param;
   (void)ctx;
   InitializeCriticalSection(&compat_pool_lock);
   for (int i = 0; i < COMPAT_SOCK_MAX; i++)
      compat_ep_fds[i] = -1;
   return TRUE;
}

static void
compat_pool_enter(void)
{
   InitOnceExecuteOnce(&compat_pool_once, compat_pool_init_once, NULL, NULL);
   EnterCriticalSection(&compat_pool_lock);
}

static void
compat_pool_leave(void)
{
   LeaveCriticalSection(&compat_pool_lock);
}

/* Must be called with the pool lock held. */
static int
compat_ep_alloc(void)
{
   for (int i = 0; i < COMPAT_SOCK_MAX; i++) {
      if (!compat_eps[i].in_use) {
         compat_eps[i].in_use = 1;
         compat_eps[i].refs = 0;
         compat_eps[i].peer = -1;
         compat_eps[i].event = CreateEventW(NULL, TRUE, FALSE, NULL);
         return i;
      }
   }
   return -1;
}

/* Must be called with the pool lock held. */
static int
compat_fd_alloc(int ep)
{
   for (int i = 0; i < COMPAT_SOCK_MAX; i++) {
      if (compat_ep_fds[i] < 0) {
         compat_ep_fds[i] = ep;
         compat_eps[ep].refs++;
         return COMPAT_SOCK_BASE + i;
      }
   }
   return -1;
}

/* fd -> endpoint index or -1 (needs the pool lock). */
static int
compat_fd_lookup(int fd)
{
   const int slot = fd - COMPAT_SOCK_BASE;
   if (slot < 0 || slot >= COMPAT_SOCK_MAX)
      return -1;
   return compat_ep_fds[slot];
}

/* Drop one reference; perform the endpoint shutdown when it was the last one.
 * Must be called with the pool lock held.  Returns 1 if the endpoint was
 * fully released. */
static int
compat_ep_dropref(int ep)
{
   compat_endpoint *e = &compat_eps[ep];
   if (--e->refs <= 0) {
      CloseHandle(e->rx);
      CloseHandle(e->tx);
      CloseHandle(e->event);
      DeleteCriticalSection(&e->tx_lock);
      if (e->peer >= 0 && compat_eps[e->peer].in_use)
         SetEvent(compat_eps[e->peer].event);
      e->in_use = 0;
      return 1;
   }
   return 0;
}

static BOOL
compat_pipe_read_exact(HANDLE h, void *buf, DWORD len)
{
   DWORD off = 0;
   while (off < len) {
      DWORD got;
      if (!ReadFile(h, (BYTE *)buf + off, len - off, &got, NULL)) {
         errno = EIO;
         return FALSE;
      }
      if (got == 0) {
         errno = EIO;
         return FALSE;
      }
      off += got;
   }
   return TRUE;
}

static BOOL
compat_pipe_write_exact(HANDLE h, const void *buf, DWORD len)
{
   DWORD off = 0;
   while (off < len) {
      DWORD wrote;
      if (!WriteFile(h, (const BYTE *)buf + off, len - off, &wrote, NULL)) {
         fprintf(stderr, "compat: pipe write failed len=%lu off=%lu GLE=%lu\n",
                 (unsigned long)len, (unsigned long)off,
                 (unsigned long)GetLastError());
         errno = EPIPE;
         return FALSE;
      }
      if (wrote == 0) {
         errno = EPIPE;
         return FALSE;
      }
      off += wrote;
   }
   return TRUE;
}

/*
 * unistd
 */

int
close(int fd)
{
   const int slot = fd - COMPAT_SOCK_BASE;

   compat_pool_enter();
   if (slot >= 0 && slot < COMPAT_SOCK_MAX && compat_ep_fds[slot] >= 0) {
      const int ep = compat_ep_fds[slot];
      compat_ep_fds[slot] = -1;
      fprintf(stderr, "compat: close virtual fd %d (refs %d)\n", fd,
              compat_eps[ep].refs);
      compat_ep_dropref(ep);
      compat_pool_leave();
      return 0;
   }
   compat_pool_leave();
   return _close(fd);
}

int
read(int fd, void *buf, unsigned int count)
{
   return _read(fd, buf, count);
}

int
write(int fd, const void *buf, unsigned int count)
{
   return _write(fd, buf, count);
}

long
lseek(int fd, long offset, int whence)
{
   return (long)_lseeki64(fd, (__int64)offset, whence);
}

int
fsync(int fd)
{
   return _commit(fd);
}

int
ftruncate(int fd, off_t length)
{
   if (_chsize_s(fd, (__int64)length) == 0)
      return 0;
   return -1;
}

int
dup(int fd)
{
   return _dup(fd);
}

int
dup2(int fd, int fd2)
{
   return _dup2(fd, fd2);
}

int
pipe(int fds[2])
{
   return _pipe(fds, 4096, _O_BINARY);
}

int
access(const char *path, int mode)
{
   return _access(path, mode);
}

int
unlink(const char *path)
{
   return _unlink(path);
}

int
rmdir(const char *path)
{
   return _rmdir(path);
}

int
chdir(const char *path)
{
   return _chdir(path);
}

char *
getcwd(char *buf, int size)
{
   return _getcwd(buf, size);
}

int
getpid(void)
{
   return _getpid();
}

long
sysconf(int name)
{
   if (name == _SC_PAGESIZE)
      return 4096;
   return -1;
}

unsigned int
sleep(unsigned int seconds)
{
   Sleep(seconds * 1000);
   return 0;
}

int
usleep(useconds_t usec)
{
   Sleep(usec / 1000);
   return 0;
}

int
setenv(const char *name, const char *value, int overwrite)
{
   char *existing = getenv(name);
   if (existing && !overwrite)
      return 0;
   if (_putenv_s(name, value) != 0)
      return -1;
   return 0;
}

int
unsetenv(const char *name)
{
   if (_putenv_s(name, "") != 0)
      return -1;
   return 0;
}

/* ------------------------------------------------------------------ */
/* sys/uio                                                            */
/* ------------------------------------------------------------------ */

ssize_t
readv(int fd, const struct iovec *iov, int iovcnt)
{
   ssize_t total = 0;
   for (int i = 0; i < iovcnt; i++) {
      ssize_t ret = read(fd, iov[i].iov_base, iov[i].iov_len);
      if (ret < 0) {
         if (total)
            return total;
         return -1;
      }
      total += ret;
      if ((size_t)ret != iov[i].iov_len)
         break;
   }
   return total;
}

ssize_t
writev(int fd, const struct iovec *iov, int iovcnt)
{
   ssize_t total = 0;
   for (int i = 0; i < iovcnt; i++) {
      ssize_t ret = write(fd, iov[i].iov_base, iov[i].iov_len);
      if (ret < 0) {
         if (total)
            return total;
         return -1;
      }
      total += ret;
      if ((size_t)ret != iov[i].iov_len)
         break;
   }
   return total;
}

/* ------------------------------------------------------------------ */
/* sys/mman                                                           */
/* ------------------------------------------------------------------ */

/* Track whether a mapping is a file-backed view (must be released with
 * UnmapViewOfFile) or a plain VirtualAlloc region (VirtualFree). */
typedef struct compat_mmap_entry {
   void *addr;
   size_t len;
   int is_view;
} compat_mmap_entry;

static compat_mmap_entry compat_mmaps[256];
static int compat_mmap_count;

static void
compat_mmap_record(void *addr, size_t len, int is_view)
{
   if (compat_mmap_count < (int)(sizeof(compat_mmaps) / sizeof(compat_mmaps[0]))) {
      compat_mmaps[compat_mmap_count].addr = addr;
      compat_mmaps[compat_mmap_count].len = len;
      compat_mmaps[compat_mmap_count].is_view = is_view;
      compat_mmap_count++;
   }
}

static compat_mmap_entry *
compat_mmap_find(void *addr)
{
   for (int i = 0; i < compat_mmap_count; i++) {
      if (compat_mmaps[i].addr == addr)
         return &compat_mmaps[i];
   }
   return NULL;
}

void *
mmap(void *addr, size_t length, int prot, int flags, int fd, off_t offset)
{
   DWORD flProtect;

   if (prot & PROT_WRITE)
      flProtect = prot & PROT_EXEC ? PAGE_EXECUTE_READWRITE : PAGE_READWRITE;
   else if (prot & PROT_READ)
      flProtect = prot & PROT_EXEC ? PAGE_EXECUTE_READ : PAGE_READONLY;
   else if (prot & PROT_EXEC)
      flProtect = PAGE_EXECUTE;
   else
      flProtect = PAGE_NOACCESS;

   if (fd >= 0 && !(flags & MAP_ANON)) {
      /* File-backed mapping: map the CRT fd's real handle so multiple
       * mmap()s of the same fd (e.g. the render server's shared timeline
       * buffer) actually share memory. */
      HANDLE h = (HANDLE)_get_osfhandle(fd);
      if (h != INVALID_HANDLE_VALUE) {
         ULARGE_INTEGER sz;
         HANDLE section;
         void *p;
         DWORD access;
         LARGE_INTEGER off_large;

         sz.QuadPart = (LONGLONG)length;
         section = CreateFileMappingW(h, NULL, flProtect, sz.HighPart, sz.LowPart,
                                      NULL);
         if (!section)
            return MAP_FAILED;
         access = (prot & PROT_WRITE) ? FILE_MAP_WRITE : FILE_MAP_READ;
         off_large.QuadPart = (LONGLONG)offset;
         p = MapViewOfFile(section, access, off_large.HighPart, off_large.LowPart,
                           length);
         CloseHandle(section);
         if (!p)
            return MAP_FAILED;
         compat_mmap_record(p, length, 1);
         return p;
      }
   }

   {
      void *p = VirtualAlloc(addr, length, MEM_RESERVE | MEM_COMMIT, flProtect);
      if (!p)
         return MAP_FAILED;
      compat_mmap_record(p, length, 0);
      return p;
   }
}

int
munmap(void *addr, size_t length)
{
   compat_mmap_entry *e = compat_mmap_find(addr);
   if (e) {
      BOOL ok;
      if (e->is_view)
         ok = UnmapViewOfFile(addr);
      else
         ok = VirtualFree(addr, 0, MEM_RELEASE);
      *e = compat_mmaps[--compat_mmap_count];
      return ok ? 0 : -1;
   }
   (void)length;
   return VirtualFree(addr, 0, MEM_RELEASE) ? 0 : -1;
}

int
mprotect(void *addr, size_t length, int prot)
{
   DWORD flProtect;
   DWORD old;

   if (prot & PROT_WRITE)
      flProtect = prot & PROT_EXEC ? PAGE_EXECUTE_READWRITE : PAGE_READWRITE;
   else if (prot & PROT_READ)
      flProtect = prot & PROT_EXEC ? PAGE_EXECUTE_READ : PAGE_READONLY;
   else if (prot & PROT_EXEC)
      flProtect = PAGE_EXECUTE;
   else
      flProtect = PAGE_NOACCESS;

   return VirtualProtect(addr, length, flProtect, &old) ? 0 : -1;
}

int
msync(void *addr, size_t length, int flags)
{
   (void)addr;
   (void)length;
   (void)flags;
   return 0;
}

int
madvise(void *addr, size_t length, int advice)
{
   (void)addr;
   (void)length;
   (void)advice;
   return 0;
}

/* ------------------------------------------------------------------ */
/* time / clock                                                       */
/* ------------------------------------------------------------------ */

int
clock_gettime(int clock_id, struct timespec *tp)
{
   union {
      FILETIME ft;
      ULARGE_INTEGER li;
   } u;
   static const unsigned long long epoch_diff = 116444736000000000ULL;

   (void)clock_id;
   GetSystemTimeAsFileTime(&u.ft);
   u.li.QuadPart -= epoch_diff;
   tp->tv_sec = (time_t)(u.li.QuadPart / 10000000ULL);
   tp->tv_nsec = (long)((u.li.QuadPart % 10000000ULL) * 100);
   return 0;
}

int
clock_nanosleep(int clock_id, int flags, const struct timespec *req,
                struct timespec *rem)
{
   struct timespec now;
   DWORD ms;

   (void)rem;
   if (flags == TIMER_ABSTIME) {
      unsigned long long target_ns;
      unsigned long long now_ns;
      clock_gettime(clock_id, &now);
      target_ns = (unsigned long long)req->tv_sec * 1000000000ULL +
                  req->tv_nsec;
      now_ns = (unsigned long long)now.tv_sec * 1000000000ULL + now.tv_nsec;
      if (target_ns <= now_ns)
         return 0;
      ms = (DWORD)((target_ns - now_ns + 999999ULL) / 1000000ULL);
      Sleep(ms);
      return 0;
   }

   ms = (DWORD)(req->tv_sec * 1000U + (req->tv_nsec / 1000000U));
   if (req->tv_nsec % 1000000U)
      ms++;
   Sleep(ms);
   return 0;
}

/* ------------------------------------------------------------------ */
/* sys/resource                                                       */
/* ------------------------------------------------------------------ */

int
setpriority(int which, id_t who, int prio)
{
   (void)which;
   (void)who;
   (void)prio;
   return 0;
}

int
getpriority(int which, id_t who)
{
   (void)which;
   (void)who;
   return 0;
}

int
getrlimit(int resource, struct rlimit *rlim)
{
   (void)resource;
   rlim->rlim_cur = 0;
   rlim->rlim_max = 0;
   return -1;
}

int
setrlimit(int resource, const struct rlimit *rlim)
{
   (void)resource;
   (void)rlim;
   return -1;
}

int
getrusage(int who, struct rusage *usage)
{
   (void)who;
   memset(usage, 0, sizeof(*usage));
   return 0;
}

/* ------------------------------------------------------------------ */
/* dlfcn                                                              */
/* ------------------------------------------------------------------ */

void *
dlopen(const char *filename, int flag)
{
   (void)flag;
   return LoadLibraryA(filename);
}

void *
dlsym(void *handle, const char *symbol)
{
   return (void *)GetProcAddress((HMODULE)handle, symbol);
}

int
dlclose(void *handle)
{
   return FreeLibrary((HMODULE)handle) ? 0 : -1;
}

char *
dlerror(void)
{
   return "dynamic linking error";
}

/* ------------------------------------------------------------------ */
/* getopt / getopt_long                                               */
/* ------------------------------------------------------------------ */

char *optarg = NULL;
int optind = 1;
int opterr = 1;
int optopt = 0;
int optreset = 0;

static int optpos = 1;

static void
compat_getopt_err(const char *argv0, const char *msg)
{
   if (opterr && argv0)
      fprintf(stderr, "%s: %s\n", argv0, msg);
}

static int
compat_match_long(const struct option *longopts, const char *longname,
                  size_t namelen, int *longindex)
{
   int i;
   for (i = 0; longopts && longopts[i].name; i++) {
      if (strlen(longopts[i].name) == namelen &&
          strncmp(longopts[i].name, longname, namelen) == 0) {
         if (longindex)
            *longindex = i;
         return i;
      }
   }
   return -1;
}

static int
compat_handle_long(int argc, char *const argv[], const struct option *longopts,
                   int *longindex, const char *arg)
{
   const char *longname = arg + 2;
   const char *eq = strchr(longname, '=');
   size_t namelen = eq ? (size_t)(eq - longname) : strlen(longname);
   int idx = compat_match_long(longopts, longname, namelen, longindex);

   if (idx < 0) {
      optopt = 0;
      compat_getopt_err(argv[0], "unrecognized option");
      optind++;
      return '?';
   }

   optind++;

   switch (longopts[idx].has_arg) {
   case required_argument:
      optarg = eq ? (char *)eq + 1 : NULL;
      if (!optarg) {
         if (optind < argc)
            optarg = argv[optind++];
         else {
            optopt = longopts[idx].val;
            compat_getopt_err(argv[0], "option requires an argument");
            return '?';
         }
      }
      break;
   case optional_argument:
      optarg = eq ? (char *)eq + 1 : NULL;
      break;
   default:
      optarg = NULL;
      if (eq) {
         compat_getopt_err(argv[0], "option doesn't allow an argument");
         return '?';
      }
      break;
   }

   if (longopts[idx].flag) {
      *longopts[idx].flag = longopts[idx].val;
      return 0;
   }
   return longopts[idx].val;
}

int
getopt(int argc, char *const argv[], const char *optstring)
{
   static const struct option empty[] = { { NULL, 0, NULL, 0 } };
   return getopt_long(argc, argv, optstring, empty, NULL);
}

int
getopt_long(int argc, char *const argv[], const char *optstring,
            const struct option *longopts, int *longindex)
{
   const char *arg;
   const char *optp;
   char c;
   int ret;

   if (optreset) {
      optreset = 0;
      optpos = 1;
      optind = 1;
      optarg = NULL;
   }

   if (optind >= argc)
      return -1;

   arg = argv[optind];
   if (arg[0] != '-' || arg[1] == '\0')
      return -1;
   if (arg[1] == '-' && arg[2] == '\0') {
      optind++;
      return -1;
   }
   if (arg[1] == '-' && longopts)
      return compat_handle_long(argc, argv, longopts, longindex, arg);

   if (optpos == 1) {
      optpos = 1;
      optarg = NULL;
   }

   c = arg[optpos];
   if (c == '\0') {
      optpos = 1;
      optind++;
      return getopt_long(argc, argv, optstring, longopts, longindex);
   }

   optopt = (unsigned char)c;
   optpos++;

   optp = strchr(optstring, c);
   if (!optp) {
      compat_getopt_err(argv[0], "illegal option");
      return '?';
   }

   if (optp[1] == ':') {
      /* option takes an argument */
      if (arg[optpos] != '\0') {
         optarg = (char *)&arg[optpos];
         optpos = 1;
         optind++;
      } else {
         optind++;
         if (optp[2] == ':') {
            optarg = NULL; /* optional argument */
         } else {
            if (optind >= argc) {
               compat_getopt_err(argv[0], "option requires an argument");
               return '?';
            }
            optarg = argv[optind++];
         }
      }
   } else {
      optarg = NULL;
      if (arg[optpos] == '\0') {
         optpos = 1;
         optind++;
      }
   }

   return (unsigned char)c;
}

/* ------------------------------------------------------------------ */
/* Windows-unused POSIX (separate-process render server / proxy)      */
/* ------------------------------------------------------------------ */

static void
compat_unused_abort(const char *fn)
{
   fprintf(stderr, "webrogue: %s: not supported on Windows\n", fn);
   abort();
}

/* sockets: the in-process socketpair transport the same-process render
 * server relies on.  SCM_RIGHTS fd passing follows Linux semantics: sending
 * an fd holds a duplicate reference (surviving the sender's close), and the
 * receiver takes the fd over.  Plain file fds are duplicated with _dup(). */
int
socketpair(int domain, int type, int protocol, int sv[2])
{
   SECURITY_ATTRIBUTES sa;
   HANDLE r1 = NULL, w1 = NULL, r2 = NULL, w2 = NULL;
   int a, b;
   int t;

   (void)domain;
   (void)protocol;

   compat_pool_enter();
   a = compat_ep_alloc();
   if (a < 0) {
      compat_pool_leave();
      errno = EMFILE;
      return -1;
   }
   b = compat_ep_alloc();
   if (b < 0) {
      compat_eps[a].in_use = 0;
      compat_pool_leave();
      errno = EMFILE;
      return -1;
   }

   sa.nLength = sizeof(sa);
   sa.bInheritHandle = TRUE;
   sa.lpSecurityDescriptor = NULL;
   if (!CreatePipe(&r1, &w1, &sa, 0) || !CreatePipe(&r2, &w2, &sa, 0)) {
      if (r1)
         CloseHandle(r1);
      if (w1)
         CloseHandle(w1);
      if (r2)
         CloseHandle(r2);
      if (w2)
         CloseHandle(w2);
      compat_eps[a].in_use = 0;
      compat_eps[b].in_use = 0;
      compat_pool_leave();
      errno = EIO;
      return -1;
   }

   /* endpoint a reads pipe1 and writes pipe2, endpoint b the other way */
   t = type & ~(SOCK_CLOEXEC | SOCK_NONBLOCK);
   compat_eps[a].rx = r1;
   compat_eps[a].tx = w2;
   compat_eps[a].type = t;
   compat_eps[a].peer = b;
   InitializeCriticalSection(&compat_eps[a].tx_lock);

   compat_eps[b].rx = r2;
   compat_eps[b].tx = w1;
   compat_eps[b].type = t;
   compat_eps[b].peer = a;
   InitializeCriticalSection(&compat_eps[b].tx_lock);

   sv[0] = compat_fd_alloc(a);
   sv[1] = compat_fd_alloc(b);
   fprintf(stderr, "compat: socketpair type=%d -> %d, %d\n", type, sv[0], sv[1]);
   compat_pool_leave();

   return sv[0] >= 0 && sv[1] >= 0 ? 0 : -1;
}

ssize_t
sendmsg(int fd, const struct msghdr *msg, int flags)
{
   struct {
      uint32_t magic;
      uint32_t len;
      uint32_t fd_count;
      int fds[COMPAT_SOCK_MAX_FD];
   } hdr;
   size_t payload_len = 0;
   BOOL ok;
   compat_endpoint *e;
   int ep;

   (void)flags;
   if (!msg || !msg->msg_iov) {
      errno = EBADF;
      return -1;
   }

   compat_pool_enter();
   ep = compat_fd_lookup(fd);
   if (ep < 0) {
      compat_pool_leave();
      errno = EBADF;
      return -1;
   }
   e = &compat_eps[ep];
   e->refs++; /* keep the endpoint alive across the blocking write */

   memset(&hdr, 0, sizeof(hdr));
   for (size_t i = 0; i < msg->msg_iovlen; i++)
      payload_len += msg->msg_iov[i].iov_len;

   if (msg->msg_control && msg->msg_controllen >= sizeof(struct cmsghdr)) {
      const struct cmsghdr *cmsg = CMSG_FIRSTHDR(msg);
      if (cmsg && cmsg->cmsg_level == SOL_SOCKET && cmsg->cmsg_type == SCM_RIGHTS) {
         int n = (int)((cmsg->cmsg_len - CMSG_LEN(0)) / sizeof(int));
         if (n < 0)
            n = 0;
         if (n > COMPAT_SOCK_MAX_FD)
            n = COMPAT_SOCK_MAX_FD;
         hdr.fd_count = (uint32_t)n;
         for (int i = 0; i < n; i++) {
            const int src_fd = ((const int *)CMSG_DATA(cmsg))[i];
            int src_ep = compat_fd_lookup(src_fd);
            if (src_ep >= 0) {
               /* duplicate: hand the receiver a fresh fd slot so the sender's
                * later close() cannot invalidate it */
               int dup_fd = compat_fd_alloc(src_ep);
               hdr.fds[i] = dup_fd >= 0 ? dup_fd : src_fd;
            } else {
               hdr.fds[i] = src_fd;
            }
         }
      }
   }

   hdr.magic = COMPAT_SOCK_MAGIC;
   hdr.len = (uint32_t)payload_len;

   EnterCriticalSection(&e->tx_lock);
   ok = compat_pipe_write_exact(e->tx, &hdr, sizeof(hdr));
   for (size_t i = 0; ok && i < msg->msg_iovlen; i++) {
      ok = compat_pipe_write_exact(e->tx, msg->msg_iov[i].iov_base,
                                   (DWORD)msg->msg_iov[i].iov_len);
   }
   LeaveCriticalSection(&e->tx_lock);

   compat_ep_dropref(ep);
   compat_pool_leave();

   if (!ok)
      return -1;
   return (ssize_t)payload_len;
}

ssize_t
recvmsg(int fd, struct msghdr *msg, int flags)
{
   struct {
      uint32_t magic;
      uint32_t len;
      uint32_t fd_count;
      int fds[COMPAT_SOCK_MAX_FD];
   } hdr;
   size_t remaining, delivered;
   HANDLE rx;
   int ep;
   size_t i = 0;
   BOOL eof;
   int out_fds[COMPAT_SOCK_MAX_FD];

   (void)flags;
   (void)eof;
   if (!msg || !msg->msg_iov) {
      errno = EBADF;
      return -1;
   }

   compat_pool_enter();
   ep = compat_fd_lookup(fd);
   if (ep < 0) {
      compat_pool_leave();
      errno = EBADF;
      return -1;
   }
   compat_eps[ep].refs++;
   rx = compat_eps[ep].rx;
   compat_pool_leave();

   if (!compat_pipe_read_exact(rx, &hdr, sizeof(hdr))) {
      fprintf(stderr, "compat: recvmsg fd=%d header read failed GLE=%lu\n", fd,
              (unsigned long)GetLastError());
      compat_pool_enter();
      compat_ep_dropref(ep);
      compat_pool_leave();
      return 0; /* peer closed */
   }
   if (hdr.magic != COMPAT_SOCK_MAGIC) {
      errno = EPROTO;
      compat_pool_enter();
      compat_ep_dropref(ep);
      compat_pool_leave();
      return -1;
   }

   remaining = hdr.len;
   delivered = 0;
   while (remaining && i < msg->msg_iovlen) {
      size_t chunk = msg->msg_iov[i].iov_len;
      if (chunk > remaining)
         chunk = remaining;
      if (chunk && !compat_pipe_read_exact(rx, msg->msg_iov[i].iov_base,
                                           (DWORD)chunk)) {
         errno = EPIPE;
         msg->msg_flags |= MSG_CTRUNC;
         compat_pool_enter();
         compat_ep_dropref(ep);
         compat_pool_leave();
         return -1;
      }
      remaining -= chunk;
      delivered += chunk;
      i++;
   }

   if (remaining) {
      /* caller's buffer is too small: drain the tail and report truncation */
      char buf[4096];
      while (remaining) {
         size_t chunk = remaining > sizeof(buf) ? sizeof(buf) : remaining;
         if (!compat_pipe_read_exact(rx, buf, (DWORD)chunk))
            break;
         remaining -= chunk;
      }
      msg->msg_flags |= MSG_TRUNC;
   } else {
      msg->msg_flags = 0;
   }

   compat_pool_enter();
   for (uint32_t f = 0; f < hdr.fd_count; f++) {
      int src = hdr.fds[f];
      if (src >= 0 && src < COMPAT_SOCK_BASE) {
         /* plain file fd: give the receiver its own copy */
         int ndup = _dup(src);
         out_fds[f] = ndup >= 0 ? ndup : src;
      } else {
         /* virtual fd: already duplicated at send time, take it over */
         out_fds[f] = src;
      }
   }
   compat_ep_dropref(ep);
   compat_pool_leave();

   if (hdr.fd_count) {
      if (msg->msg_control &&
          msg->msg_controllen >= CMSG_SPACE((size_t)hdr.fd_count * sizeof(int))) {
         struct cmsghdr *cmsg = (struct cmsghdr *)msg->msg_control;
         cmsg->cmsg_len = CMSG_LEN((size_t)hdr.fd_count * sizeof(int));
         cmsg->cmsg_level = SOL_SOCKET;
         cmsg->cmsg_type = SCM_RIGHTS;
         memcpy(CMSG_DATA(cmsg), out_fds, (size_t)hdr.fd_count * sizeof(int));
         msg->msg_controllen = CMSG_SPACE((size_t)hdr.fd_count * sizeof(int));
      } else {
         msg->msg_flags |= MSG_CTRUNC;
         msg->msg_controllen = 0;
      }
   } else {
      msg->msg_controllen = 0;
   }

   return (ssize_t)delivered;
}

int
getsockopt(int fd, int level, int optname, void *optval, socklen_t *optlen)
{
   int ep;
   int type;
   int so_error = 0;
   int out;

   compat_pool_enter();
   ep = compat_fd_lookup(fd);
   if (ep < 0) {
      compat_pool_leave();
      errno = ENOTSOCK;
      return -1;
   }
   type = compat_eps[ep].type;
   compat_pool_leave();

   if (level != SOL_SOCKET) {
      errno = EOPNOTSUPP;
      return -1;
   }
   if (optname == SO_TYPE || optname == SO_ERROR) {
      if (optval && optlen && *optlen >= sizeof(int)) {
         out = optname == SO_TYPE ? type : so_error;
         *(int *)optval = out;
         *optlen = sizeof(int);
         return 0;
      }
      errno = EINVAL;
      return -1;
   }
   errno = EOPNOTSUPP;
   return -1;
}

int
setsockopt(int fd, int level, int optname, const void *optval, socklen_t optlen)
{
   int ep;
   (void)level;
   (void)optname;
   (void)optval;
   (void)optlen;

   compat_pool_enter();
   ep = compat_fd_lookup(fd);
   compat_pool_leave();
   if (ep < 0) {
      errno = ENOTSOCK;
      return -1;
   }
   return 0;
}

/* poll */
int
poll(struct pollfd *fds, nfds_t nfds, int timeout)
{
   HANDLE handles[MAXIMUM_WAIT_OBJECTS];
   int ep_of_handle[MAXIMUM_WAIT_OBJECTS];
   DWORD nhandles = 0;
   ULONGLONG deadline = 0;
   int finite;
   int plain_ready = 0;

   if (!nfds)
      return 0;

   compat_pool_enter();
   for (nfds_t i = 0; i < nfds; i++) {
      int ep;
      fds[i].revents = 0;
      ep = compat_fd_lookup(fds[i].fd);
      if (ep >= 0) {
         if (nhandles + 2 <= MAXIMUM_WAIT_OBJECTS) {
            compat_eps[ep].refs++; /* keep endpoint alive while waiting */
            handles[nhandles] = compat_eps[ep].rx;
            ep_of_handle[nhandles] = ep;
            nhandles++;
            handles[nhandles] = compat_eps[ep].event;
            ep_of_handle[nhandles] = ep;
            nhandles++;
         }
      } else {
         /* not a virtual socket: assume ready */
         fds[i].revents = (short)(fds[i].events & (POLLIN | POLLOUT));
         if (!fds[i].revents)
            fds[i].revents = POLLOUT;
         plain_ready++;
      }
   }
   compat_pool_leave();

   if (nhandles == 0) {
      if (timeout >= 0)
         Sleep((DWORD)timeout);
      else
         Sleep(INFINITE);
      return plain_ready;
   }

   if (plain_ready) {
      /* an always-ready fd: return immediately, like poll(2) */
      compat_pool_enter();
      for (DWORD i = 0; i < nhandles; i += 2)
         compat_ep_dropref(ep_of_handle[i]);
      compat_pool_leave();
      return plain_ready;
   }

   finite = timeout >= 0;
   if (finite)
      deadline = GetTickCount64() + (ULONGLONG)timeout;

   for (;;) {
      DWORD ms;
      ULONGLONG now;
      DWORD w;

      if (finite) {
         now = GetTickCount64();
         if (now >= deadline)
            break;
         ms = (DWORD)(deadline - now);
      } else {
         ms = INFINITE;
      }

      w = WaitForMultipleObjects(nhandles, handles, FALSE, ms);
      if (w == WAIT_FAILED) {
         fprintf(stderr, "compat: poll fd=%d nhandles=%lu WAIT_FAILED GLE=%lu\n",
                 fds[0].fd, (unsigned long)nhandles,
                 (unsigned long)GetLastError());
         errno = EINVAL;
         break;
      }
      if (w == WAIT_TIMEOUT)
         break;

      {
         int ready = plain_ready;
         compat_pool_enter();
         for (nfds_t i = 0; i < nfds; i++) {
            int ep = compat_fd_lookup(fds[i].fd);
            if (ep >= 0) {
               DWORD avail = 0;
               if (PeekNamedPipe(compat_eps[ep].rx, NULL, 0, NULL, &avail, NULL)) {
                  if (avail > 0) {
                     fds[i].revents = (short)(POLLIN | POLLRDNORM);
                     ready++;
                  } else if (WaitForSingleObject(compat_eps[ep].event, 0) ==
                             WAIT_OBJECT_0) {
                     fds[i].revents = (short)(POLLERR | POLLHUP);
                     ready++;
                  } else {
                     /* spurious wakeup with no data: keep waiting */
                  }
               } else {
                  fds[i].revents = (short)(POLLERR | POLLHUP);
                  ready++;
               }
            }
         }
         compat_pool_leave();
         if (ready)
            break;
      }
   }

   compat_pool_enter();
   for (DWORD i = 0; i < nhandles; i += 2)
      compat_ep_dropref(ep_of_handle[i]);
   compat_pool_leave();

   {
      int ready = 0;
      for (nfds_t i = 0; i < nfds; i++) {
         if (fds[i].revents)
            ready++;
      }
      return ready;
   }
}

int
socket(int domain, int type, int protocol)
{
   (void)domain;
   (void)type;
   (void)protocol;
   compat_unused_abort("socket");
   return -1;
}

int
connect(int fd, const struct sockaddr *addr, socklen_t len)
{
   (void)fd;
   (void)addr;
   (void)len;
   compat_unused_abort("connect");
   return -1;
}

int
bind(int fd, const struct sockaddr *addr, socklen_t len)
{
   (void)fd;
   (void)addr;
   (void)len;
   compat_unused_abort("bind");
   return -1;
}

int
listen(int fd, int backlog)
{
   (void)fd;
   (void)backlog;
   compat_unused_abort("listen");
   return -1;
}

int
accept(int fd, struct sockaddr *addr, socklen_t *len)
{
   (void)fd;
   (void)addr;
   (void)len;
   compat_unused_abort("accept");
   return -1;
}

ssize_t
send(int fd, const void *buf, size_t len, int flags)
{
   (void)fd;
   (void)buf;
   (void)len;
   (void)flags;
   compat_unused_abort("send");
   return -1;
}

ssize_t
recv(int fd, void *buf, size_t len, int flags)
{
   (void)fd;
   (void)buf;
   (void)len;
   (void)flags;
   compat_unused_abort("recv");
   return -1;
}

int
shutdown(int fd, int how)
{
   (void)fd;
   (void)how;
   compat_unused_abort("shutdown");
   return -1;
}

/* ioctl (dma-buf fencing, Linux-only) */
int
ioctl(int fd, unsigned long request, ...)
{
   (void)fd;
   (void)request;
   compat_unused_abort("ioctl");
   return -1;
}

/* signalfd */
int
signalfd(int fd, const sigset_t *mask, int flags)
{
   (void)fd;
   (void)mask;
   (void)flags;
   compat_unused_abort("signalfd");
   return -1;
}

/* wait */
pid_t
wait(int *status)
{
   (void)status;
   compat_unused_abort("wait");
   return -1;
}

pid_t
waitpid(pid_t pid, int *status, int options)
{
   (void)pid;
   (void)status;
   (void)options;
   compat_unused_abort("waitpid");
   return -1;
}

int
waitid(idtype_t idtype, id_t id, siginfo_t *infop, int options)
{
   (void)idtype;
   (void)id;
   (void)infop;
   (void)options;
   compat_unused_abort("waitid");
   return -1;
}
