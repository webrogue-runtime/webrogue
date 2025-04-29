set -ex

cd $(dirname $0)

docker --version | grep podman >/dev/null || {
    DOCKER_USER_FLAGS="-u $(id -u ${USER}):$(id -g ${USER})"
}
image_name=webrogue_logo_converter
docker build -t $image_name -f convert.Dockerfile .
echo $(dirname $(dirname $(pwd)))
docker run \
    -v $(dirname $(dirname $(pwd))):/host_dir \
    $DOCKER_USER_FLAGS \
    -t $image_name
