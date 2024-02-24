set -ex

cd $(dirname $0)

image_name=deb2appimage
docker build -t $image_name -f deb2appimage.Dockerfile .
id=$(docker create $image_name)
docker cp $id:Webrogue-x86_64.AppImage - | tar -x
docker rm -v $id
