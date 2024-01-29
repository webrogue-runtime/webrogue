# set(
#     WABT_SOURCES 
    
#     ${WEBROGUE_ROOT_PATH}/external/wabt/src/linking/compact_linker.cc

#     ${WEBROGUE_ROOT_PATH}/external/wabt/src/apply-names.cc
#     ${WEBROGUE_ROOT_PATH}/external/wabt/src/binary-reader-ir.cc
#     ${WEBROGUE_ROOT_PATH}/external/wabt/src/binary-reader-logging.cc
#     ${WEBROGUE_ROOT_PATH}/external/wabt/src/binary-reader.cc
#     ${WEBROGUE_ROOT_PATH}/external/wabt/src/binary-writer-spec.cc
#     ${WEBROGUE_ROOT_PATH}/external/wabt/src/binary-writer.cc
#     ${WEBROGUE_ROOT_PATH}/external/wabt/src/binary.cc
#     ${WEBROGUE_ROOT_PATH}/external/wabt/src/binding-hash.cc
#     ${WEBROGUE_ROOT_PATH}/external/wabt/src/color.cc
#     ${WEBROGUE_ROOT_PATH}/external/wabt/src/common.cc
#     ${WEBROGUE_ROOT_PATH}/external/wabt/src/config.cc
#     ${WEBROGUE_ROOT_PATH}/external/wabt/src/config.h.in
#     ${WEBROGUE_ROOT_PATH}/external/wabt/src/decompiler.cc
#     ${WEBROGUE_ROOT_PATH}/external/wabt/src/error-formatter.cc
#     ${WEBROGUE_ROOT_PATH}/external/wabt/src/expr-visitor.cc
#     ${WEBROGUE_ROOT_PATH}/external/wabt/src/feature.cc
#     ${WEBROGUE_ROOT_PATH}/external/wabt/src/filenames.cc
#     ${WEBROGUE_ROOT_PATH}/external/wabt/src/generate-names.cc
#     ${WEBROGUE_ROOT_PATH}/external/wabt/src/ir-util.cc
#     ${WEBROGUE_ROOT_PATH}/external/wabt/src/ir.cc
#     ${WEBROGUE_ROOT_PATH}/external/wabt/src/leb128.cc
#     ${WEBROGUE_ROOT_PATH}/external/wabt/src/lexer-source-line-finder.cc
#     ${WEBROGUE_ROOT_PATH}/external/wabt/src/lexer-source.cc
#     ${WEBROGUE_ROOT_PATH}/external/wabt/src/literal.cc
#     ${WEBROGUE_ROOT_PATH}/external/wabt/src/opcode-code-table.c
#     ${WEBROGUE_ROOT_PATH}/external/wabt/src/opcode.cc
#     ${WEBROGUE_ROOT_PATH}/external/wabt/src/option-parser.cc
#     ${WEBROGUE_ROOT_PATH}/external/wabt/src/resolve-names.cc
#     ${WEBROGUE_ROOT_PATH}/external/wabt/src/sha256.cc
#     ${WEBROGUE_ROOT_PATH}/external/wabt/src/shared-validator.cc
#     ${WEBROGUE_ROOT_PATH}/external/wabt/src/stream.cc
#     ${WEBROGUE_ROOT_PATH}/external/wabt/src/token.cc
#     ${WEBROGUE_ROOT_PATH}/external/wabt/src/tracing.cc
#     ${WEBROGUE_ROOT_PATH}/external/wabt/src/type-checker.cc
#     ${WEBROGUE_ROOT_PATH}/external/wabt/src/utf8.cc
#     ${WEBROGUE_ROOT_PATH}/external/wabt/src/validator.cc
#     ${WEBROGUE_ROOT_PATH}/external/wabt/src/wast-lexer.cc
#     ${WEBROGUE_ROOT_PATH}/external/wabt/src/wast-parser.cc
#     ${WEBROGUE_ROOT_PATH}/external/wabt/src/wat-writer.cc
#     # ${WEBROGUE_ROOT_PATH}/external/wabt/src/c-writer.cc

#     ${WEBROGUE_ROOT_PATH}/src/core/CompactLinking.cpp
# )

# add_library(
#     compact_linker SHARED
    
#     ${WABT_SOURCES}
# )

# if (MSVC)
#   set(COMPILER_IS_CLANG 0)
#   set(COMPILER_IS_GNU 0)
#   set(COMPILER_IS_MSVC 1)
# elseif (CMAKE_C_COMPILER_ID MATCHES "Clang")
#   set(COMPILER_IS_CLANG 1)
#   set(COMPILER_IS_GNU 0)
#   set(COMPILER_IS_MSVC 0)
# elseif (CMAKE_C_COMPILER_ID STREQUAL "GNU")
#   set(COMPILER_IS_CLANG 0)
#   set(COMPILER_IS_GNU 1)
#   set(COMPILER_IS_MSVC 0)
# elseif (CMAKE_SYSTEM_NAME STREQUAL "Emscripten")
#   set(COMPILER_IS_CLANG 1)
#   set(COMPILER_IS_GNU 0)
#   set(COMPILER_IS_MSVC 0)
# else ()
#   set(COMPILER_IS_CLANG 0)
#   set(COMPILER_IS_GNU 0)
#   set(COMPILER_IS_MSVC 0)
# endif ()

# set(WABT_VERSION_STRING 1.0.33)
# set(HAVE_OPENSSL_SHA_H OFF)

# configure_file(${WEBROGUE_ROOT_PATH}/external/wabt/src/config.h.in ${CMAKE_CURRENT_BINARY_DIR}/include/wabt/config.h @ONLY)

# target_include_directories(
#     compact_linker PRIVATE
#     ${CMAKE_CURRENT_BINARY_DIR}/include
#     ${WEBROGUE_ROOT_PATH}/external/wabt/include
#     ${WEBROGUE_ROOT_PATH}/external/wabt/third_party/picosha2
# )


add_subdirectory(${WEBROGUE_ROOT_PATH}/src/linker compact_linker)
