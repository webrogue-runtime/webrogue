cd "$(dirname "$0")"
set -ex

sh ../../crates/vfs/wit/update_deps.sh

rm -rf deps
mkdir deps
cp ../../crates/vfs/wit/deps/filesystem.wit deps
cp ../../crates/vfs/wit/deps/clocks.wit deps
cp ../../crates/vfs/wit/webrogue-vfs.wit deps
