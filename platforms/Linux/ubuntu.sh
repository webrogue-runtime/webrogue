set -ex

cd $(dirname $0)

image_name=webrogue_deb
docker build -t $image_name -f ubuntu.Dockerfile .
id=$(docker create $image_name)
docker cp $id:webrogue.deb - | tar -x
docker rm -v $id
