FROM rust:alpine

RUN apk add pkgconfig openssl-dev musl-dev cmake make openssl-libs-static
RUN apk add clang19 llvm19 lld
RUN apk add gcc g++
RUN apk add strace
# RUN apt-get -y install cmake llvm-16 strace clang-16 lld-16 libc++abi-16-dev libc++-16-dev
# RUN update-alternatives --install /usr/bin/llvm-ar llvm-ar /usr/bin/llvm-ar-16 1
# RUN update-alternatives --install /usr/bin/lld lld /usr/bin/lld-16 1
# RUN update-alternatives --install /usr/bin/clang clang /usr/bin/clang-16 1
# RUN update-alternatives --install /usr/bin/clang++ clang++ /usr/bin/clang++-16 1

