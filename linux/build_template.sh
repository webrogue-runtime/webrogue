
set -ex

REPO_ROOT="$(dirname $(dirname $(realpath $0)))"

cd "$REPO_ROOT"
WRAPP_PATH=examples/empty/empty.wrapp

if test -f $WRAPP_PATH
then
    cargo run --release --features=compile --no-default-features compile object $WRAPP_PATH linux/empty.gnu.o x86_64-linux-gnu --pic
    cargo run --release --features=compile --no-default-features compile object $WRAPP_PATH linux/empty.musl.o x86_64-linux-musl --pic
fi

cd "$REPO_ROOT/linux/bookworm"
IMAGE_NAME=webrogue/webrogue-linux-gnu-builder
docker build --tag $IMAGE_NAME .
docker run \
    --rm \
    --user "$(id -u)":"$(id -g)" \
    -v "$REPO_ROOT":/usr/src/myapp \
    -w /usr/src/myapp \
    $IMAGE_NAME \
    sh linux/bookworm/build_template.sh

cd "$REPO_ROOT/linux/alpine"
IMAGE_NAME=webrogue/webrogue-linux-musl-builder
docker build --tag $IMAGE_NAME .
docker run \
    --rm \
    --user "$(id -u)":"$(id -g)" \
    -v "$REPO_ROOT":/usr/src/myapp \
    -w /usr/src/myapp \
    $IMAGE_NAME \
    sh linux/alpine/build_template.sh
