file(
    DOWNLOAD "https://github.com/libsdl-org/SDL/releases/download/prerelease-2.29.2/SDL2-devel-2.29.2-VC.zip" ${CMAKE_BINARY_DIR}/SDL2-VC.zip
    EXPECTED_HASH SHA512=5ca9ef35de18a14bdbd1090e97164609cca4f1f21984efccd61ce6b055dcb09a7de096afa278690728c9fbce03e039d72638a3ed459fcd1e52b536da94a9ec43
    SHOW_PROGRESS
)

execute_process(COMMAND ${CMAKE_COMMAND} -E tar xfz ${CMAKE_BINARY_DIR}/SDL2-VC.zip
    WORKING_DIRECTORY ${CMAKE_BINARY_DIR}
    RESULT_VARIABLE rv)
if(NOT rv EQUAL 0)
    message(FATAL_ERROR "error: extraction of '${_filename}' failed")
endif()

file(
    DOWNLOAD "https://github.com/libsdl-org/SDL_ttf/releases/download/release-2.22.0/SDL2_ttf-devel-2.22.0-VC.zip" ${CMAKE_BINARY_DIR}/SDL2_ttf-VC.zip
    EXPECTED_HASH SHA512=8de7bf679be1f2834fd88877838965412645a7875b4c6ecbed27815ad2252687de77c586a62e87157f23293f9af4123e96fa271e3f6249e49d70edb2f13b755f
    SHOW_PROGRESS
)

execute_process(COMMAND ${CMAKE_COMMAND} -E tar xfz ${CMAKE_BINARY_DIR}/SDL2_ttf-VC.zip
    WORKING_DIRECTORY ${CMAKE_BINARY_DIR}
    RESULT_VARIABLE rv)
if(NOT rv EQUAL 0)
    message(FATAL_ERROR "error: extraction of '${_filename}' failed")
endif()

file(GLOB SDL2_INCLUDE_DIR ${CMAKE_BINARY_DIR}/SDL2-*/include/)
file(GLOB SDL2_TTF_INCLUDE_DIR ${CMAKE_BINARY_DIR}/SDL2_ttf-*/include)
file(GLOB SDL2_LIBRARY ${CMAKE_BINARY_DIR}/SDL2-*/lib/x64/SDL2.lib)
file(GLOB SDL2_TTF_LIBRARY ${CMAKE_BINARY_DIR}/SDL2_ttf-*/lib/x64/SDL2_ttf.lib)

file(GLOB SDL2_DLLS ${CMAKE_BINARY_DIR}/SDL2-*/lib/x64/SDL2.dll)
file(GLOB SDL2_TTF_DLLS ${CMAKE_BINARY_DIR}/SDL2_ttf-*/lib/x64/SDL2_ttf.dll)
