
set -ex

REPO_ROOT="$(dirname $(dirname $(realpath $0)))"
# ARCHES="x86_64 aarch64"
ARCHES=$(uname -m)

cd "$REPO_ROOT"
WRAPP_PATH=examples/empty/empty.wrapp

if test -f $WRAPP_PATH
then
    cargo run --release --features=compile --no-default-features compile object $WRAPP_PATH linux/empty.gnu.x86_64.o x86_64-linux-gnu
    cargo run --release --features=compile --no-default-features compile object $WRAPP_PATH linux/empty.gnu.aarch64.o aarch64-linux-gnu

    cargo run --release --features=compile --no-default-features compile object $WRAPP_PATH linux/empty.musl.x86_64.o x86_64-linux-musl --pic
    cargo run --release --features=compile --no-default-features compile object $WRAPP_PATH linux/empty.musl.aarch64.o aarch64-linux-musl --pic
fi

docker --version | grep podman >/dev/null || {
    DOCKER_USER_FLAGS="-u $(id -u ${USER}):$(id -g ${USER})"
}
cd "$REPO_ROOT/linux/glibc"
for ARCH in $ARCHES
do
    IMAGE_NAME=webrogue/webrogue-linux-gnu-$ARCH-builder
    docker build \
        --build-arg ARCH=$ARCH \
        --platform linux/$ARCH \
        --tag $IMAGE_NAME \
        .

    docker run \
        --rm \
        $DOCKER_USER_FLAGS \
        -e ARCH=$ARCH \
        --platform linux/$ARCH \
        -v "$REPO_ROOT":/usr/src/myapp \
        -w /usr/src/myapp \
        $IMAGE_NAME \
        sh linux/glibc/build_cli.sh
done

# cd "$REPO_ROOT/linux/musl"
# IMAGE_NAME=webrogue/webrogue-linux-musl-builder
# docker build --tag $IMAGE_NAME .
# docker run \
#     --rm \
#     $DOCKER_USER_FLAGS \
#     -v "$REPO_ROOT":/usr/src/myapp \
#     -w /usr/src/myapp \
#     $IMAGE_NAME \
#     sh linux/musl/build_cli.sh\
