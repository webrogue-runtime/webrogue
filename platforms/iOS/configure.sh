cd $(dirname $0)

set -ex

cmake -B cmake_build/ -S . -G Xcode -DCMAKE_SYSTEM_NAME=iOS
