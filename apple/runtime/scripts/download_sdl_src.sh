cd $(dirname $(dirname $0))
set -ex

if [ ! -f "external/SDL3.zip" ]; then
    curl -L https://codeload.github.com/libsdl-org/SDL/zip/refs/tags/release-3.2.10 -o external/SDL3.zip
fi
if [ ! -d "external/SDL3" ]; then
    tar -xf external/SDL3.zip -C external
    mv external/SDL-release-3.2.10 external/SDL3
    patch --forward external/SDL3/src/video/uikit/SDL_uikitappdelegate.m external/sdl.patch
fi

sed -i '' 's/defaultConfigurationName = Debug/defaultConfigurationName = Release/g' external/SDL3/Xcode/SDL/SDL.xcodeproj/project.pbxproj
