#ifndef _COMPAT_SYS_SOCKET_H
#define _COMPAT_SYS_SOCKET_H

#include <stddef.h>
#include <sys/types.h>
#include <sys/uio.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef unsigned short sa_family_t;
typedef int socklen_t;

#define AF_UNIX 1
#define AF_LOCAL AF_UNIX
#define PF_UNIX AF_UNIX
#define PF_LOCAL AF_UNIX

#define SOCK_STREAM 1
#define SOCK_DGRAM 2
#define SOCK_RAW 3
#define SOCK_SEQPACKET 5
#define SOCK_CLOEXEC 0
#define SOCK_NONBLOCK 0

#define SOL_SOCKET 1

#define SO_REUSEADDR 0x0004
#define SO_TYPE 0x1008
#define SO_ERROR 0x1007
#define SO_RCVTIMEO 0x1006
#define SO_SNDTIMEO 0x1005

#define SCM_RIGHTS 0x01

#define MSG_OOB 0x1
#define MSG_PEEK 0x2
#define MSG_DONTROUTE 0x4
#define MSG_TRYHARD 0x4
#define MSG_CTRUNC 0x8
#define MSG_PROBE 0x10
#define MSG_TRUNC 0x20
#define MSG_DONTWAIT 0x40
#define MSG_EOR 0x80
#define MSG_WAITALL 0x100
#define MSG_NOSIGNAL 0x4000
#define MSG_CMSG_CLOEXEC 0

#define SHUT_RD 0
#define SHUT_WR 1
#define SHUT_RDWR 2

struct sockaddr {
   sa_family_t sa_family;
   char sa_data[14];
};

struct sockaddr_storage {
   sa_family_t ss_family;
   char __ss_pad[sizeof(void *) == 8 ? 120 : 112];
};

struct msghdr {
   void *msg_name;
   socklen_t msg_namelen;
   struct iovec *msg_iov;
   size_t msg_iovlen;
   void *msg_control;
   size_t msg_controllen;
   int msg_flags;
};

struct cmsghdr {
   size_t cmsg_len;
   int cmsg_level;
   int cmsg_type;
};

#define CMSG_ALIGN(len) (((len) + sizeof(size_t) - 1) & ~(sizeof(size_t) - 1))
#define CMSG_SPACE(len) (CMSG_ALIGN(sizeof(struct cmsghdr)) + CMSG_ALIGN(len))
#define CMSG_LEN(len) (CMSG_ALIGN(sizeof(struct cmsghdr)) + (len))
#define CMSG_FIRSTHDR(msgh)                                                    \
   ((msgh)->msg_controllen >= sizeof(struct cmsghdr)                           \
       ? (struct cmsghdr *)(msgh)->msg_control                                 \
       : (struct cmsghdr *)NULL)
#define CMSG_NXTHDR(msgh, cmsg)                                                \
   ((cmsg) == NULL ? CMSG_FIRSTHDR(msgh) :                                     \
                     (((char *)(cmsg) + CMSG_ALIGN((cmsg)->cmsg_len) +         \
                       CMSG_ALIGN(sizeof(struct cmsghdr)) <=                   \
                      (char *)(msgh)->msg_control + (msgh)->msg_controllen) ?  \
                         (struct cmsghdr *)((char *)(cmsg) +                   \
                                            CMSG_ALIGN((cmsg)->cmsg_len)) :    \
                         (struct cmsghdr *)NULL))
#define CMSG_DATA(cmsg)                                                        \
   ((unsigned char *)(cmsg) + CMSG_ALIGN(sizeof(struct cmsghdr)))

static inline unsigned int _compat_htons(unsigned int x)
{
   return (unsigned int)((x & 0xffU) << 8 | ((x >> 8) & 0xffU));
}

static inline unsigned long _compat_htonl(unsigned long x)
{
   return ((x & 0xffUL) << 24) | ((x & 0xff00UL) << 8) | ((x >> 8) & 0xff00UL) |
          ((x >> 24) & 0xffUL);
}

#define htons(x) _compat_htons((unsigned int)(x))
#define htonl(x) _compat_htonl((unsigned long)(x))
#define ntohs(x) _compat_htons((unsigned int)(x))
#define ntohl(x) _compat_htonl((unsigned long)(x))

/* Map the socket API to compat-unique symbol names so these declarations and
 * the stub definitions in win32_compat.c never collide with ws2_32's own
 * exports (socket, bind, send, ...) at link time. */
#define socketpair compat_socketpair
#define socket compat_socket
#define connect compat_connect
#define bind compat_bind
#define listen compat_listen
#define accept compat_accept
#define send compat_send
#define recv compat_recv
#define sendmsg compat_sendmsg
#define recvmsg compat_recvmsg
#define getsockopt compat_getsockopt
#define setsockopt compat_setsockopt
#define shutdown compat_shutdown

int socketpair(int domain, int type, int protocol, int sv[2]);
int socket(int domain, int type, int protocol);
int connect(int fd, const struct sockaddr *addr, socklen_t len);
int bind(int fd, const struct sockaddr *addr, socklen_t len);
int listen(int fd, int backlog);
int accept(int fd, struct sockaddr *addr, socklen_t *len);
ssize_t send(int fd, const void *buf, size_t len, int flags);
ssize_t recv(int fd, void *buf, size_t len, int flags);
ssize_t sendmsg(int fd, const struct msghdr *msg, int flags);
ssize_t recvmsg(int fd, struct msghdr *msg, int flags);
int getsockopt(int fd, int level, int optname, void *optval, socklen_t *optlen);
int setsockopt(int fd, int level, int optname, const void *optval,
               socklen_t optlen);
int shutdown(int fd, int how);

#ifdef __cplusplus
}
#endif

#endif
