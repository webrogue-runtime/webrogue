cd $(dirname $0)/..
set -ex

rm -rf ~/.local/share/webrogue/aot_artifacts/
cp -r aot_artifacts ~/.local/share/webrogue
