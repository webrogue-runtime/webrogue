FROM ubuntu:20.04 as builder

# Timezone fix for old images
ENV TZ=Asia/Dubai
RUN ln -snf /usr/share/zoneinfo/$TZ /etc/localtime && echo $TZ > /etc/timezone

RUN apt update
RUN apt-get install -y build-essential libncurses-dev git curl wget rpm libsdl2-dev libsdl2-ttf-dev

RUN wget https://github.com/Kitware/CMake/releases/download/v3.29.0-rc2/cmake-3.29.0-rc2-linux-x86_64.sh -O cmake_installer.sh && mkdir /opt_cmake && bash cmake_installer.sh --prefix=/opt_cmake --skip-license && ln -s /opt_cmake/bin/* /usr/local/bin && rm cmake_installer.sh

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

WORKDIR "/"
RUN git clone https://github.com/webrogue-runtime/webrogue.git
WORKDIR "/webrogue"

RUN git submodule update --init --recursive external/wasmer/ && \
    git submodule update --init --recursive external/xz/ && \
    git submodule update --init --recursive external/argparse/ && \
    git submodule update --init --recursive external/libuv/ && \
    git submodule update --init --recursive external/uvwasi/ && \
    git submodule update --init --recursive external/SDL/ && \
    git submodule update --init --recursive external/SDL_ttf/

COPY git_hash git_hash
RUN git pull && git submodule update --recursive

RUN sh platforms/Linux/build.sh

FROM ubuntu:20.04
# FROM fedora:38
COPY --from=builder /webrogue/platforms/Linux/webrogue.deb webrogue.deb
COPY --from=builder /webrogue/platforms/Linux/webrogue.rpm webrogue.rpm
RUN apt-get install -y ./webrogue.deb
# RUN rpm -i webrogue.rpm
