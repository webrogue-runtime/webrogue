cd $(dirname $0)
set -ex


test -f angle_windows_x64.zip || wget https://github.com/webrogue-runtime/angle-builder/releases/latest/download/windows_x64.zip -O angle_windows_x64.zip
test -f libEGL.dll || unzip -j angle_windows_x64.zip x64/libEGL.dll
test -f libGLESv2.dll || unzip -j angle_windows_x64.zip x64/libGLESv2.dll
