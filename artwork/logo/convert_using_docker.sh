set -ex

cd $(dirname $0)

image_name=webrogue_logo_converter
docker build -t $image_name -f convert.Dockerfile .
echo $(dirname $(dirname $(pwd)))
docker run \
    -v $(dirname $(dirname $(pwd))):/host_dir \
    -u $(id -u ${USER}):$(id -g ${USER}) \
    -t $image_name
