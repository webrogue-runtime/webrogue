cd $(dirname $0)
set -ex

IMAGE_NAME=webrogue/webrogue-linux-aot-artifacts-builder
docker build --tag $IMAGE_NAME .
docker run \
    --rm \
    --user "$(id -u)":"$(id -g)" \
    -v "$(dirname $(dirname $PWD))":/usr/src/myapp \
    -w /usr/src/myapp \
    $IMAGE_NAME \
    sh scripts/_build_linux_aot_artifacts.sh \
# || docker run \
#     -it \
#     --rm \
#     --user "$(id -u)":"$(id -g)" \
#     -v "$(dirname $(dirname $PWD))":/usr/src/myapp \
#     -w /usr/src/myapp \
#     $IMAGE_NAME \
#     bash
