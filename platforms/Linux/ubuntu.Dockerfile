ARG image=ubuntu:22.04

FROM $image as builder

# Timezone fix for old images
ENV TZ=Asia/Dubai
RUN ln -snf /usr/share/zoneinfo/$TZ /etc/localtime && echo $TZ > /etc/timezone

RUN apt update
RUN apt-get install -y build-essential libncurses-dev libsdl2-dev libsdl2-ttf-dev git curl wget

RUN wget https://github.com/Kitware/CMake/releases/download/v3.29.0-rc2/cmake-3.29.0-rc2-linux-x86_64.sh -O cmake_installer.sh && mkdir /opt_cmake && bash cmake_installer.sh --prefix=/opt_cmake --skip-license && ln -s /opt_cmake/bin/* /usr/local/bin && rm cmake_installer.sh

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

COPY src src
COPY platforms/Linux platforms/Linux
COPY cmake cmake
COPY make_webrogue.cmake make_webrogue.cmake
COPY mods mods

COPY external/wasmer/ external/wasmer/
COPY external/xz/ external/xz/
COPY external/argparse/ external/argparse/
COPY external/libuv/ external/libuv/
COPY external/uvwasi/ external/uvwasi/
COPY external/zstd/ external/zstd/
COPY external/sqlite_amb/ external/sqlite_amb/
COPY external/wasmer.patch/ external/wasmer.patch/

RUN cmake -S platforms/Linux/ -B platforms/Linux/build/ -DCMAKE_BUILD_TYPE=Release && cmake --build platforms/Linux/build/ --target webrogue --parallel && cpack --config platforms/Linux/build/CPackConfig.cmake && cp webrogue-*-Linux.deb webrogue.deb


FROM $image
COPY --from=builder /webrogue.deb webrogue.deb
RUN apt update
RUN apt-get install -y ./webrogue.deb

# SHELL [ "webrogue" ]
