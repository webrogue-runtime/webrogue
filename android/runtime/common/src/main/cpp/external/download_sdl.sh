set -ex
cd $(dirname $0)/../../../..


SDL_VERSION="$(cat src/main/cpp/external/sdl_version.txt)"

if [ ! -f src/main/cpp/external/SDL-release-$SDL_VERSION.zip ]; then
    curl https://codeload.github.com/libsdl-org/SDL/zip/refs/tags/release-$SDL_VERSION -o src/main/cpp/external/SDL-release-$SDL_VERSION.zip
fi
if [ ! -d src/main/cpp/external/SDL-release-$SDL_VERSION ]; then
    cd src/main/cpp/external
    unzip SDL-release-$SDL_VERSION.zip
    cd ../../../..

    mkdir -p src/main/java/org/libsdl
    rm -rf src/main/java/org/libsdl/app
    cp -r src/main/cpp/external/SDL-release-$SDL_VERSION/android-project/app/src/main/java/org/libsdl/app src/main/java/org/libsdl/app
fi
