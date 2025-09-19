cd $(dirname $0)/..
set -ex

cd aot_artifacts/
zip -r ../aot_artifacts .
cd ..
