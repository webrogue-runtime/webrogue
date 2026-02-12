set -ex

cd $(dirname $(realpath $0))/..
mkdir -p external
cd external

if [ ! -f "MoltenVK-all.tar" ]; then
    curl -L https://github.com/KhronosGroup/MoltenVK/releases/latest/download/MoltenVK-all.tar -o MoltenVK-all.tar
fi
if [ ! -d "MoltenVK" ]; then
    tar xf MoltenVK-all.tar
fi
