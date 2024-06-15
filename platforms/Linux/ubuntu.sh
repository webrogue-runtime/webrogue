set -ex

cd $(dirname $0)

image_name=webrogue_deb
docker build -t $image_name -f ubuntu.Dockerfile .
docker run \
  -it \
  --rm \
  -v "$(dirname $(dirname $(pwd))):/webrogue" \
  $image_name \
  bash -c "
    cd /webrogue &&\
    git config --global --add safe.directory /webrogue &&\
    git clean -d -f -x &&\
    . scripts/make_venv.sh &&\
    sh platforms/Linux/build.sh
  "
