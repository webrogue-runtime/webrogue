mod code_writer;
mod events;

#[derive(clap::Parser, Clone)]
struct Cli {
    enc_path: std::path::PathBuf,
    dec_h_path: std::path::PathBuf,
    dec_c_path: std::path::PathBuf,
}

fn main() -> anyhow::Result<()> {
    let args = <Cli as clap::Parser>::parse();

    let mut enc_writer = code_writer::CodeWriter::new();
    let mut dec_h_writer = code_writer::CodeWriter::new();
    let mut dec_c_writer = code_writer::CodeWriter::new();

    let mut writer = &mut enc_writer;
    macro_rules! write {
        ($($arg:tt)*) => {
            writer.writeln(&format!($($arg)*))?;
        };
    }
    {
        write!("#pragma once");
        write!("// GENERATED BY webrogue-event-apigen. DO NOT EDIT MANUALLY");
        write!("#include <stdint.h>");
        write!("");
        write!("typedef struct webrogue_event_out_buf {{");
        writer.inc_indent();
        {
            write!("void* buf;");
            write!("uint32_t buf_size;");
            write!("uint32_t used_size;");
        }
        writer.dec_indent();
        write!("}} webrogue_event_out_buf;");
        write!("");
        write!("#define BUF_SIZE(LEN) if(out->buf_size < LEN) {{ out->used_size = 0; return; }} out->used_size = LEN");
        write!("#define SET(TYPE, OFFSET, VALUE) *((TYPE*)(((char*)out->buf) + OFFSET)) = VALUE");
        write!("");
        for event in events::all() {
            let mut args = vec!["webrogue_event_out_buf *out".to_owned()];
            for field in event.fields.clone() {
                args.push(field.ty.c_name().to_owned() + " " + field.name);
            }
            write!(
                "static inline void webrogue_event_encode_{}({}) {{",
                event.name,
                args.join(", ")
            );
            writer.inc_indent();
            {
                write!("BUF_SIZE({});", event.size);
                write!("SET(uint32_t, 0, {});", event.id);
                for field in event.fields.clone() {
                    write!(
                        "SET({}, {}, {});",
                        field.ty.c_name(),
                        field.offset, field.name
                    );
                }
            }
            writer.dec_indent();
            write!("}}");
        }
        write!("");
        write!("#undef BUF_SIZE");
        write!("#undef SET");
    }
    writer = &mut dec_h_writer;
    {
        write!("// GENERATED BY webrogue-event-apigen. DO NOT EDIT MANUALLY");
        write!("#include <stddef.h>");
        write!("#include <stdint.h>");
        write!("");
        write!("void webrogue_gfx_present();");
        write!("void webrogue_gfx_make_window();");
        write!("void webrogue_gfx_window_size(int *width, int *height);");
        write!("void webrogue_gfx_gl_size(int *width, int *height);");
        write!("void webrogue_gfx_init_gl();");
        write!("void *webrogueGLLoader(const char *procname);");
        write!("");
        write!("// Events");
        for event in events::all() {
            write!("struct webrogue_event_{} {{", event.name);
            writer.inc_indent();
            {
                for field in event.fields.clone() {
                    write!("{} {};", field.ty.c_name(), field.name);
                }
            }
            writer.dec_indent();
            write!("}};");
        }
        write!("enum webrogue_event_type {{");
        writer.inc_indent();
        {
            write!("webrogue_event_type_invalid = 0,");
            for event in events::all() {
                write!("webrogue_event_type_{} = {},", event.name, event.id);
            }
        }
        writer.dec_indent();
        write!("}};");
        write!("typedef struct webrogue_event {{");
        writer.inc_indent();
        {
            write!("enum webrogue_event_type type;");
            write!("union {{");
            writer.inc_indent();
            {
                for event in events::all() {
                    write!("struct webrogue_event_{} {};", event.name, event.name);
                }
            }
            writer.dec_indent();
            write!("}} inner;");
        }
        writer.dec_indent();
        write!("}} webrogue_event;");
        write!("");
        write!("webrogue_event webrogue_gfx_poll();");
    }

    writer = &mut dec_c_writer;
    {
        write!("// GENERATED BY webrogue-event-apigen. DO NOT EDIT MANUALLY");
        write!("#include \"webrogue_gfx.h\"");
        write!("#include <stdlib.h>");
        write!("");
        write!("__attribute__((import_name(\"poll\")))");
        write!("__attribute__((import_module(\"webrogue-gfx\")))");
        write!("uint32_t imported_webrogue_gfx_poll();");
        write!("");
        write!("__attribute__((import_name(\"poll-read\")))");
        write!("__attribute__((import_module(\"webrogue-gfx\")))");
        write!("void imported_webrogue_gfx_poll_read(void *buf);");
        write!("#define BUF_SIZE(LEN) if(buffer_len < LEN) {{\\");
        writer.inc_indent();
        {
            write!("free(buffer_data);\\");
            write!("result.type = webrogue_event_type_invalid;\\");
            write!("return result;\\");
        }
        writer.dec_indent();
        write!("}}");
        write!("#define RETURN free(buffer_data); return result;");
        write!("#define GET(TYPE, OFFSET) *((TYPE*)(((char*)buffer_data) + OFFSET));");
        write!("");
        write!("webrogue_event webrogue_gfx_poll() {{");
        writer.inc_indent();
        {
            write!("webrogue_event result;");
            write!("uint32_t buffer_len = imported_webrogue_gfx_poll();");
            write!("void* buffer_data = malloc(buffer_len);");
            write!("BUF_SIZE(4);");
            write!("imported_webrogue_gfx_poll_read(buffer_data);");
            write!("result.type = GET(uint32_t, 0);");
            write!("switch (result.type) {{");
            writer.inc_indent();
            {
                for event in events::all() {
                    write!("case webrogue_event_type_{}: {{", event.name);
                    writer.inc_indent();
                    {
                        write!("BUF_SIZE({});", event.size);
                        for field in event.fields.clone() {
                            write!("result.inner.{}.{} = GET({}, {});", event.name, field.name, field.ty.c_name(), field.offset);
                        }
                        write!("RETURN;");
                    }
                    writer.dec_indent();
                    write!("}}");
                }
            }
            writer.dec_indent();

            writer.inc_indent();
            {
                write!("default: {{");
                writer.inc_indent();
                {
                    write!("result.type = webrogue_event_type_invalid;");
                    write!("RETURN;");
                }
                writer.dec_indent();
                write!("}}");
            }
            writer.dec_indent();
            write!("}}");
        }
        writer.dec_indent();
        write!("}}");
    }
    write!("#undef BUF_SIZE");
    write!("#undef RETURN");
    write!("#undef GET");

    enc_writer.write_to_file(&args.enc_path)?;
    dec_h_writer.write_to_file(&args.dec_h_path)?;
    dec_c_writer.write_to_file(&args.dec_c_path)?;

    anyhow::Ok(())
}
