ARG ARCH
FROM quay.io/pypa/musllinux_1_2_${ARCH}
ARG ARCH

RUN apk add pkgconfig openssl-dev musl-dev cmake make openssl-libs-static clang19 llvm19 lld gcc g++ strace bzip2-dev

ENV PATH="/fakehome/.cargo/bin:$PATH"
ENV RUSTUP_HOME=/fakehome/rustup
ENV CARGO_HOME=/fakehome/.cargo
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | HOME=/fakehome sh -s -- -y --no-modify-path

RUN mkdir -p /fakehome/.cargo/registry/cache \
    /fakehome/.cargo/registry/index \
    /fakehome/.cargo/registry/src \
    /fakehome/.cargo/git/db
RUN chmod -R a+rwx /fakehome/.cargo/

# SDL deps
RUN apk add pipewire-dev alsa-lib-dev pulseaudio-dev libx11-dev libx11-static \
    libxext-dev libxrandr-dev libxcursor-dev libxfixes-dev libxi-dev \
    libxscrnsaver-dev dbus-dev ibus-dev openpgl-dev mesa-dev \
    libxkbcommon-dev wayland-dev libdrm-dev libusb-dev \
    liburing-dev pipewire-jack-dev libdecor-dev bzip2-static xz-dev xz-static

RUN apk add pkgconfig openssl-dev musl-dev cmake make openssl-libs-static clang llvm lld gcc g++ strace bzip2-dev

# RUN rm /usr/lib/libX11.so  

ENV ARCH=${ARCH}
