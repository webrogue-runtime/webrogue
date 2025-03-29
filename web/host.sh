cd $(dirname $0)
set -ex

sh build.sh

cd root
npx --yes http-server
