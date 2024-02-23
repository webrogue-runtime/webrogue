ARG image=ubuntu:22.04

FROM $image as builder

# Timezone fix for old images
ENV TZ=Asia/Dubai
RUN ln -snf /usr/share/zoneinfo/$TZ /etc/localtime && echo $TZ > /etc/timezone

RUN apt update
RUN apt-get install -y build-essential libncurses-dev libsdl2-dev libsdl2-ttf-dev git curl wget

RUN wget https://github.com/Kitware/CMake/releases/download/v3.29.0-rc2/cmake-3.29.0-rc2-linux-x86_64.sh -O cmake_installer.sh && mkdir /opt_cmake && bash cmake_installer.sh --prefix=/opt_cmake --skip-license && ln -s /opt_cmake/bin/* /usr/local/bin && rm cmake_installer.sh

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

WORKDIR "/"
RUN git clone https://github.com/webrogue-runtime/webrogue.git
WORKDIR "/webrogue"

RUN git submodule update --init --recursive --single-branch external/wasmer/ && \
    git submodule update --init --recursive --single-branch external/xz/ && \
    git submodule update --init --recursive --single-branch external/argparse/ && \
    git submodule update --init --recursive --single-branch external/libuv/ && \
    git submodule update --init --recursive --single-branch external/uvwasi/ && \
    git submodule update --init --recursive --single-branch external/SDL/ && \
    git submodule update --init --recursive --single-branch external/SDL_ttf/

RUN cmake -S platforms/Linux/ -B platforms/Linux/build/ -DCMAKE_BUILD_TYPE=Release -DWEBROGUE_APPIMAGE=OFF
RUN cmake --build platforms/Linux/build/ --target webrogue --parallel 
RUN cpack --config platforms/Linux/build/CPackConfig.cmake && cp webrogue-*-Linux.deb webrogue.deb

RUN cmake -S . -B build/ -DCMAKE_BUILD_TYPE=Release -DWEBROGUE_APPIMAGE=ON
RUN cmake --build build/ --target webrogue --parallel 
RUN cmake --install build/ --prefix .


FROM $image
COPY --from=builder /webrogue/webrogue.deb webrogue.deb
COPY --from=builder /webrogue/App.AppImage App.AppImage
RUN apt update
RUN apt-get install -y ./webrogue.deb

# SHELL [ "webrogue" ]
