cd $(dirname $0)

configure_and_build() {
    echo configure_and_build
    CC=cl CXX=cl cmake -B build -S . -DCMAKE_BUILD_TYPE=Release -DCMAKE_SYSTEM_NAME=Windows && \
        cmake --build build --target webrogue --target webrogue --parallel || exit 1
}

configure_and_build || \
    configure_and_build || \
    configure_and_build || \
    exit 1
