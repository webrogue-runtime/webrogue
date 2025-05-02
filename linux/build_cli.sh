
set -ex

REPO_ROOT="$(dirname $(dirname $(realpath $0)))"

cd "$REPO_ROOT"
WRAPP_PATH=examples/empty/empty.wrapp

if test -f $WRAPP_PATH
then
    cargo run --release --features=compile --no-default-features compile object $WRAPP_PATH linux/empty.gnu.o x86_64-linux-gnu --pic
    cargo run --release --features=compile --no-default-features compile object $WRAPP_PATH linux/empty.musl.o x86_64-linux-musl --pic
fi

docker --version | grep podman >/dev/null || {
    DOCKER_USER_FLAGS="-u $(id -u ${USER}):$(id -g ${USER})"
}
cd "$REPO_ROOT/linux/glibc"
IMAGE_NAME=webrogue/webrogue-linux-gnu-builder
docker build --tag $IMAGE_NAME .
docker run \
    --rm \
    $DOCKER_USER_FLAGS \
    -v "$REPO_ROOT":/usr/src/myapp \
    -w /usr/src/myapp \
    $IMAGE_NAME \
    sh linux/glibc/build_cli.sh
# || docker run \
#     -it \
#     --rm \
#     --user "$(id -u)":"$(id -g)" \
#     -v "$(dirname $(dirname $PWD))":/usr/src/myapp \
#     -w /usr/src/myapp \
#     $IMAGE_NAME \
#     bash
