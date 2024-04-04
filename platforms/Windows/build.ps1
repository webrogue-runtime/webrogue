powershell.exe -noprofile -c "rm -r -fo webrogue.msix; exit 0"
powershell.exe -noprofile -c "rm -r -fo .\artifacts\; exit 0"
mkdir .\artifacts\
$BuildConfig = $args[0]
echo "Build config: $BuildConfig"
cmake -S platforms/Windows/ -B platforms/Windows/build -DCMAKE_MSVC_RUNTIME_LIBRARY=MultiThreadedDLL
cmake --build platforms/Windows/build -t webrogue_windows --config $BuildConfig --parallel
cmake --install .\platforms\Windows\build\ --prefix artifacts --config $BuildConfig
cp .\platforms\Windows\AppxManifest.xml .\artifacts\
cp -Recurse .\platforms\Windows\Images\ .\artifacts\
makeappx pack /d .\artifacts\ /p webrogue.msix
cpack --config .\platforms\Windows\build/CPackConfig.cmake
mv webrogue-*-win64.exe artifacts/webrogue_installer.exe
rm -r -fo _CPack_Packages
