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

# set MACH_O_TYPE = staticlib
cat external/SDL3/Xcode/SDL/SDL.xcodeproj/project.pbxproj | tr '\n' '\f' > external/SDL3/Xcode/SDL/SDL.xcodeproj/project.pbxproj2
cat external/SDL3/Xcode/SDL/SDL.xcodeproj/project.pbxproj2 | sed -e 's/\t\t\t\tCLANG_LINK_OBJC_RUNTIME = NO;\t\t\t\tOTHER_LDFLAGS = "-liconv";/\t\t\t\tCLANG_LINK_OBJC_RUNTIME = NO;\t\t\t\tMACH_O_TYPE = staticlib;\t\t\t\tOTHER_LDFLAGS = "-liconv";/g' | tr '\f' '\n' > external/SDL3/Xcode/SDL/SDL.xcodeproj/project.pbxproj
rm -f external/SDL3/Xcode/SDL/SDL.xcodeproj/project.pbxproj2
