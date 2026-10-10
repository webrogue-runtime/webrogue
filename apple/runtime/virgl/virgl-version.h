#ifndef VIRGL_VERSION_H
#define VIRGL_VERSION_H

#define VIRGL_MAJOR_VERSION 1
#define VIRGL_MINOR_VERSION 3
#define VIRGL_MICRO_VERSION 0

#define VIRGL_CHECK_VERSION(major, minor, micro) \
   (VIRGL_MAJOR_VERSION > (major) || \
    VIRGL_MAJOR_VERSION == (major) && VIRGL_MINOR_VERSION > (minor) || \
    VIRGL_MAJOR_VERSION == (major) && VIRGL_MINOR_VERSION == (minor) && VIRGL_MICRO_VERSION >= (micro))

#endif /* VIRGL_VERSION_H */
