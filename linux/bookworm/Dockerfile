FROM rust:bookworm

RUN apt update
RUN apt-get -y install cmake llvm-19 strace clang-19 lld-19 libc++abi-19-dev libc++-19-dev
RUN update-alternatives --install /usr/bin/llvm-ar llvm-ar /usr/bin/llvm-ar-19 1
RUN update-alternatives --install /usr/bin/lld lld /usr/bin/lld-19 1
RUN update-alternatives --install /usr/bin/clang clang /usr/bin/clang-19 1
RUN update-alternatives --install /usr/bin/clang++ clang++ /usr/bin/clang++-19 1
