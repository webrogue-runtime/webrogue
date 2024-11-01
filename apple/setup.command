cd $(dirname $0)
set -ex

sh scripts/download_angle_ios_headers.sh
sh scripts/download_angle_ios_xcframeworks.sh
sh scripts/download_sdl_src.sh

xcodegen -c
