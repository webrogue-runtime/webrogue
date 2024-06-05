set -ex

cd $(dirname $0)

image_name=deb2appimage
docker build -t $image_name -f deb2appimage.Dockerfile .
id=$(docker create $image_name)
docker cp $id:webrogue-x86_64.AppImage webrogue-x86_64.AppImage
docker rm -v $id
