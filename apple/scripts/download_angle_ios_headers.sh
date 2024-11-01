set -ex

cd $(dirname $0)/../external

if [ ! -f "angle_ios_headers.zip" ]; then
    wget https://github.com/webrogue-runtime/angle-builder/releases/latest/download/ios_headers.zip -O angle_ios_headers.zip
fi
if [ ! -d "angle_ios_headers" ]; then
    tar -xf angle_ios_headers.zip
    mv ios_headers angle_ios_headers
fi
