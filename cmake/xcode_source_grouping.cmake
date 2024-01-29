#from https://stackoverflow.com/questions/31652255/source-group-cmake-command-isnt-working

function (target_source_group)
  set (_options
    GROUP_INTERFACE_SOURCES
    )
  set (_multi_value_args
    # Required
    TARGET
    ROOT_DIR
    )
  set (_one_value_args
    PREFIX
    )

  cmake_parse_arguments (i
    "${_options}" "${_one_value_args}" "${_multi_value_args}" ${ARGN})

  # Check inputs

  foreach (_target IN LISTS i_TARGET)
    if (i_GROUP_INTERFACE_SOURCES)
      get_target_property (_target_sources ${_target} INTERFACE_SOURCES)
    else ()
      get_target_property (_target_sources ${_target} SOURCES)
    endif ()

    # Remove sources to be installed
    set (_source_to_install_regex
      "(\\$<INSTALL_INTERFACE:([^>;<$]+)>)")

    string (REGEX REPLACE
      "${_source_to_install_regex}"
      ""
      _sources_to_build
      "${_target_sources}")

    # Remove remaining ";"s. It seems safer to do it this way rather than include
    # them in _source_to_install_regex
    string (REGEX REPLACE
      "[;]+"
      ";"
      _sources_to_build
      "${_sources_to_build}")

    # Extract sources to be built
    set (_source_to_build_regex
      "\\$<BUILD_INTERFACE:([^>;<$]+)>")

    string (REGEX REPLACE
      "${_source_to_build_regex}"
      "\\1"
      _sources_to_build
      "${_sources_to_build}")

    foreach (_root IN LISTS i_ROOT_DIR)
      set (_sources_under_root_regex
        "${_root}/[^>;<$]+")

      string (REGEX MATCHALL
        "${_sources_under_root_regex}"
        _sources_under_root
        "${_sources_to_build}")

      source_group (
        TREE    "${_root}"
        FILES   ${_sources_under_root}
        PREFIX  "${i_PREFIX}"
        )
    endforeach ()
  endforeach ()
endfunction (target_source_group)
