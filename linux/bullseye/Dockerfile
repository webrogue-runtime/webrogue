FROM rust:bullseye

RUN apt update
RUN apt-get -y install cmake llvm-16 strace clang-16 lld-16 libc++abi-16-dev libc++-16-dev
RUN update-alternatives --install /usr/bin/llvm-ar llvm-ar /usr/bin/llvm-ar-16 1
RUN update-alternatives --install /usr/bin/lld lld /usr/bin/lld-16 1
RUN update-alternatives --install /usr/bin/clang clang /usr/bin/clang-16 1
RUN update-alternatives --install /usr/bin/clang++ clang++ /usr/bin/clang++-16 1

