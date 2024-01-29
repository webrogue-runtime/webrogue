set(
    WABT_SOURCES 
    
    compact-linker.cc
    archive-reader.cc
    synthesizer.cc
    synthesizer/function-synthesizer.cc
    synthesizer/import-synthesizer.cc

    binary-reader-ir.cc
    binary-reader.cc
    binary-writer.cc
    binary.cc
    binding-hash.cc
    common.cc
    config.cc
    feature.cc
    ir-util.cc
    ir.cc
    leb128.cc
    literal.cc
    opcode-code-table.c
    opcode.cc
    stream.cc
    utf8.cc
)

if(${WEBROGUE_COMPACT_LINKER_SHARED})
    add_library(compact_linker SHARED ${WABT_SOURCES})
elseif(${WEBROGUE_COMPACT_LINKER_STATIC})
    add_library(compact_linker STATIC ${WABT_SOURCES})
else()
    message(FATAL_ERROR "WEBROGUE_COMPACT_LINKER_SHARED or WEBROGUE_COMPACT_LINKER_STATIC must be ON")
endif()
