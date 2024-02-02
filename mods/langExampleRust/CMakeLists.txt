add_library(langExampleRust INTERFACE)

set(PRODUCT ${CMAKE_CURRENT_SOURCE_DIR}/target/wasm32-wasi/release/libhello_wasm.a)

include(ExternalProject)
ExternalProject_Add(
	langExampleRust-crate
	SOURCE_DIR ${CMAKE_CURRENT_SOURCE_DIR}
	DOWNLOAD_COMMAND ""
	CONFIGURE_COMMAND ""
	INSTALL_COMMAND "${WASMER_INSTALL_COMMAND}"
    BUILD_COMMAND cargo build -v --target wasm32-wasi --release
	BINARY_DIR ${CMAKE_CURRENT_SOURCE_DIR}/target
    BUILD_ALWAYS ON
	BUILD_BYPRODUCTS ${PRODUCT}
)

add_dependencies(langExampleRust langExampleRust-crate)
target_link_libraries(langExampleRust INTERFACE ${PRODUCT})

register_mod(
    NAME langExampleRust
    ARCHIVE_PATH ${PRODUCT}
)
