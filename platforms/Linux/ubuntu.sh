set -ex

cd $(dirname $0)

image_name=webrogue_deb
git rev-parse --verify HEAD > git_hash
docker build -t $image_name -f ubuntu.Dockerfile .
id=$(docker create $image_name)
docker cp $id:webrogue.deb - | tar -x
docker cp $id:webrogue.rpm - | tar -x
docker rm -v $id
