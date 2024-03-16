powershell.exe -noprofile -c "rm -r -fo .\artifacts\; exit 0"
mkdir .\artifacts\
$BuildConfig = $args[0]
echo "Build config: $BuildConfig"
cmake -S platforms/Windows/ -B platforms/Windows/build -DCMAKE_MSVC_RUNTIME_LIBRARY=MultiThreadedDLL
cmake --build platforms/Windows/build -t webrogue_windows --config $BuildConfig --parallel
cmake --install .\platforms\Windows\build\ --prefix artifacts
cpack --config .\platforms\Windows\build/CPackConfig.cmake
mv webrogue-*-win64.exe artifacts/webrogue_installer.exe
rm -r -fo _CPack_Packages
