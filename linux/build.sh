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
            cargo run --release --features=internal-compile --no-default-features internal compile object $WRAPP_PATH linux/empty.$LIBC.$ARCH.o $ARCH-linux-$LIBC
        done
    done
fi

docker --version | grep podman >/dev/null || {
    DOCKER_USER_FLAGS="-u $(id -u ${USER}):$(id -g ${USER})"
}


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
            
            MOUNT_ROOT="$REPO_ROOT/linux/mounts"
        
            mkdir -p "$MOUNT_ROOT/cargo_cache"
            mkdir -p "$MOUNT_ROOT/cargo_index"
            mkdir -p "$MOUNT_ROOT/cargo_src"
            mkdir -p "$MOUNT_ROOT/cargo_git_db"
            mkdir -p "$MOUNT_ROOT/cargo_git_checkouts"

            docker run \
                --rm \
                $DOCKER_USER_FLAGS \
                -v "$REPO_ROOT":/usr/src/myapp \
                -v "$MOUNT_ROOT/cargo_cache":/fakehome/.cargo/registry/cache \
                -v "$MOUNT_ROOT/cargo_index":/fakehome/.cargo/registry/index \
                -v "$MOUNT_ROOT/cargo_src":/fakehome/.cargo/registry/src \
                -v "$MOUNT_ROOT/cargo_git_db":/fakehome/.cargo/git/db \
                -v "$MOUNT_ROOT/cargo_git_checkouts":/fakehome/.cargo/git/checkouts \
                -w /usr/src/myapp \
                $IMAGE_NAME \
                sh linux/$LIBC/build_$COMPONENT.sh
        done
    done
done
