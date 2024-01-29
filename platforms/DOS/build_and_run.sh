cmake --toolchain=djgpp_toolchain.cmake -S platforms/DOS/ -B platforms/DOS/build -DCMAKE_BUILD_TYPE=Debug || exit $?
cmake --build platforms/DOS/build --target pack_executable_to_artifacts -j || exit $?
cp -r artifacts/* ~/dosgame/ || exit $?
dosbox -c "mount G artifacts" -c "G:" -c "WEBROGUE.EXE" || exit $?
