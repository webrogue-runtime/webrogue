ARG ARCH
FROM quay.io/pypa/manylinux_2_28_${ARCH}
ARG ARCH

# https://github.com/libsdl-org/SDL/blob/main/docs/README-linux.md#build-dependencies
# following libs are missing! pipewire-jack-audio-connection-kit-devel libdecor-devel nas-devel
RUN dnf -y install gcc git-core make cmake \
    alsa-lib-devel pulseaudio-libs-devel pipewire-devel \
    libX11-devel libXext-devel libXrandr-devel libXcursor-devel libXfixes-devel \
    libXi-devel libXScrnSaver-devel dbus-devel ibus-devel \
    systemd-devel mesa-libGL-devel libxkbcommon-devel mesa-libGLES-devel \
    mesa-libEGL-devel vulkan-devel wayland-devel wayland-protocols-devel \
    libdrm-devel mesa-libgbm-devel libusb-devel \
    liburing-devel


RUN dnf -y install llvm strace clang lld openssl-devel

ENV PATH="/fakehome/.cargo/bin:$PATH"
ENV RUSTUP_HOME=/fakehome/rustup
ENV CARGO_HOME=/fakehome/.cargo
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | HOME=/fakehome sh -s -- -y --no-modify-path
RUN chmod -R a+rwx /fakehome/.cargo/

ENV GCC_VERSION=14
RUN mv /opt/rh/gcc-toolset-$GCC_VERSION /opt/rh/current-gcc-toolset \
    && rm -rf /opt/rh/gcc-toolset-* \
    && mv /opt/rh/current-gcc-toolset /opt/rh/gcc-toolset-$GCC_VERSION

ENV ARCH=${ARCH}
