set(LLVM_TARGETS_TO_BUILD "X86;AArch64;RISCV;LoongArch")
set(LLVM_TARGETS_TO_BUILD "X86;AArch64;RISCV;LoongArch" CACHE STRING "Semicolon-separated list of targets to build, or \"all\".")

set(LLVM_ENABLE_TERMINFO "OFF")
set(LLVM_ENABLE_TERMINFO "OFF" CACHE STRING "Use terminfo database if available.")

add_subdirectory(../../external/llvm/llvm llvm EXCLUDE_FROM_ALL)

project(webrogue_linker CXX)

add_library(lldAsLib STATIC lldAsLib.cpp)

set(libs lldCommon lldCOFF lldELF lldMachO lldMinGW lldWasm LLVMExecutionEngine) #  LLVMSupport LLVMCodeGen LLVMCore LLVMDebugInfoDWARF LLVMDemangle LLVMMC LLVMOption LLVMTarget LLVMTargetParser

target_link_libraries(lldAsLib ${libs})
target_include_directories(
    lldAsLib PRIVATE
    ${CMAKE_CURRENT_SOURCE_DIR}/../../external/llvm/lld/include
    ${CMAKE_CURRENT_SOURCE_DIR}/../../external/llvm/llvm/include
    ${CMAKE_CURRENT_BINARY_DIR}/llvm/include
)

set(LIBS_TO_LINK)

function(search_for_libs_to_link lib_name out_list trace all_so_far)
    get_target_property(LINK_LIBRARIES ${lib_name} LINK_LIBRARIES)
    # message(WARNING "LINK_LIBRARIES for ${lib_name}: ${LINK_LIBRARIES}")
    # message("trace=${trace}")

    list(LENGTH all_so_far all_so_far_length)
    # message("all_so_far_length=${all_so_far_length}")

    # needs optimization
    set(result ${all_so_far})
    if(NOT LINK_LIBRARIES STREQUAL LINK_LIBRARIES-NOTFOUND)
        foreach(LINK_LIB IN ITEMS ${LINK_LIBRARIES})
            if (TARGET ${LINK_LIB})
                list(FIND result ${LINK_LIB} duplicate_index)
                if(${duplicate_index} EQUAL -1)
                    # message(WARNING "LINK_LIB in ${lib_name}: ${LINK_LIB}")
                    set(child_result)
                    search_for_libs_to_link("${LINK_LIB}" child_result "${trace}->${LINK_LIB}" "${result}")
                    set(result ${child_result} ${LINK_LIB})
                endif()
            endif()
        endforeach()
    endif()
    set(${out_list} ${result} PARENT_SCOPE)
endfunction()

search_for_libs_to_link(lldCOFF LIBS_TO_LINK lldCOFF "${LIBS_TO_LINK}")
search_for_libs_to_link(lldELF LIBS_TO_LINK lldELF "${LIBS_TO_LINK}")
search_for_libs_to_link(lldMachO LIBS_TO_LINK lldMachO "${LIBS_TO_LINK}")
search_for_libs_to_link(lldMinGW LIBS_TO_LINK lldMinGW "${LIBS_TO_LINK}")
search_for_libs_to_link(lldWasm LIBS_TO_LINK lldWasm "${LIBS_TO_LINK}")
search_for_libs_to_link(LLVMExecutionEngine LIBS_TO_LINK LLVMExecutionEngine "${LIBS_TO_LINK}")

message("LIBS_TO_LINK=${LIBS_TO_LINK}")
list(APPEND LIBS_TO_LINK lldELF lldMachO lldMinGW lldWasm lldAsLib LLVMExecutionEngine)
message("LIBS_TO_LINK=${LIBS_TO_LINK}")
list(REVERSE LIBS_TO_LINK)
message("LIBS_TO_LINK=${LIBS_TO_LINK}")

file(WRITE ${CMAKE_CURRENT_BINARY_DIR}/lldAsLib_deps.txt "${LIBS_TO_LINK}")
# foreach(LINK_LIB IN ITEMS ${LIBS_TO_LINK})
#     execute_process(COMMAND ${CMAKE_COMMAND} -E echo "cargo:rustc-link-lib=static=${LINK_LIB}")
# endforeach()

list(REMOVE_DUPLICATES LIBS_TO_LINK)
install(TARGETS ${LIBS_TO_LINK})
