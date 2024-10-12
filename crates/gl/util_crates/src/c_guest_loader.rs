use crate::types::*;
use std::io::Write;

pub fn write_to_file(file: &mut std::fs::File, parse_results: &ParseResults) {
    file.write(
        r#"#include <GLES2/gl2.h>
#define GL_GLEXT_PROTOTYPES
#include <GLES2/gl2ext.h>
#include <GLES3/gl3.h>
#include <stddef.h>
#include <string.h>

// clang-format off

__attribute__((import_name("init_ptrs")))
__attribute__((import_module("webrogue_gl"))) void
imported_init_ptrs();

void* webrogueGLLoader(const char* procname) {
  static char loaded = 0;
  if(!loaded) {
    imported_init_ptrs();
    loaded = 1;
  }
"#
        .as_bytes(),
    )
    .unwrap();

    for command in parse_results.commands.clone() {
        file.write(
            format!(
                r#"  if (strcmp(procname, "{}") == 0)
    return (void *){};
"#,
                command.name, command.name,
            )
            .as_bytes(),
        )
        .unwrap();
    }
    file.write(
        r#"  return NULL;
}
        "#
        .as_bytes(),
    )
    .unwrap();
}
