set -ex

REPO_ROOT="$(dirname $(dirname $(realpath $0)))"
# ARCHES="x86_64 aarch64"
ARCHES=$(uname -m)
# LIBCS="gnu musl"
# COMPONENTS="template cli"

cd "$REPO_ROOT"
WRAPP_PATH=webrogue-sdk/examples/empty/empty.wrapp

if test -f $WRAPP_PATH
then
    for LIBC in gnu musl
    do
        for ARCH in x86_64 aarch64
        do
            cargo run --release --features=compile --no-default-features compile object $WRAPP_PATH linux/empty.$LIBC.$ARCH.o $ARCH-linux-$LIBC
        done
    done
fi

docker --version | grep podman >/dev/null || {
    DOCKER_USER_FLAGS="-u $(id -u ${USER}):$(id -g ${USER})"
}

mkdir -p "$REPO_ROOT/linux/mounts/cargo_cache"
mkdir -p "$REPO_ROOT/linux/mounts/cargo_index"
mkdir -p "$REPO_ROOT/linux/mounts/cargo_git_db"
mkdir -p "$REPO_ROOT/linux/mounts/cargo_git_checkouts"

for LIBC in $LIBCS
do
    for ARCH in $ARCHES
    do
        for COMPONENT in $COMPONENTS
        do
            cd "$REPO_ROOT/linux/$LIBC"
            IMAGE_NAME=webrogue/webrogue-linux-$LIBC-$ARCH-builder
            docker build \
                --build-arg ARCH=$ARCH \
                --platform linux/$ARCH \
                --tag $IMAGE_NAME \
                .

            docker run \
                --rm \
                $DOCKER_USER_FLAGS \
                -v "$REPO_ROOT":/usr/src/myapp \
                -v "$REPO_ROOT/linux/mounts/cargo_cache":/fakehome/.cargo/registry/cache \
                -v "$REPO_ROOT/linux/mounts/cargo_index":/fakehome/.cargo/registry/index \
                -v "$REPO_ROOT/linux/mounts/cargo_git_db":/fakehome/.cargo/git/db \
                -v "$REPO_ROOT/linux/mounts/cargo_git_checkouts":/fakehome/.cargo/git/checkouts \
                -w /usr/src/myapp \
                $IMAGE_NAME \
                sh linux/$LIBC/build_$COMPONENT.sh
        done
    done
done
