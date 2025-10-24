cd $(dirname $0)/..
set -ex

rm -f aot_artifacts.zip
cd aot_artifacts/
zip -r ../aot_artifacts .
cd ..
