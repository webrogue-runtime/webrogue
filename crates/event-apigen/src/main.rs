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
        write!("// clang-format off");
        write!("#include <stdint.h>");
        write!("#include <stdlib.h>");
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
        write!(
            "#define WEBROGUE_MAX_ENCODED_EVENT_SIZE {}",
            events::all().iter().map(|event| event.size).max().unwrap()
        );
        write!("");
        write!("static webrogue_event_out_buf webrogue_event_out_buf_create() {{");
        writer.inc_indent();
        {
            write!("webrogue_event_out_buf result;");
            write!("result.buf = malloc(WEBROGUE_MAX_ENCODED_EVENT_SIZE);");
            write!("result.buf_size = WEBROGUE_MAX_ENCODED_EVENT_SIZE;");
            write!("result.used_size = 0;");
        }
        writer.dec_indent();
        write!("}}");
        write!("");
        write!("static void webrogue_event_out_buf_delete(webrogue_event_out_buf buf) {{");
        writer.inc_indent();
        {
            write!("free(buf.buf);");
        }
        writer.dec_indent();
        write!("}}");
        write!("");
        write!("#define BUF_SIZE(LEN) if(out->buf_size < (out->used_size + LEN)) {{ out->buf_size *= 2; out->buf = realloc(out->buf, out->buf_size); }} out->used_size += LEN");
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
            write!("WEBROGUE_EVENT_TYPE_INVALID = 0,");
            for event in events::all() {
                write!(
                    "WEBROGUE_EVENT_TYPE_{} = {},",
                    event.name.to_uppercase(),
                    event.id
                );
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
        write!("// clang-format off");
        write!("#include \"webrogue_gfx.h\"");
        write!("#include <stdlib.h>");
        write!("");
        write!(
            "#define WEBROGUE_MAX_ENCODED_EVENT_SIZE {}",
            events::all().iter().map(|event| event.size).max().unwrap()
        );
        write!("");
        write!("__attribute__((import_name(\"poll\")))");
        write!("__attribute__((import_module(\"webrogue_gfx\")))");
        write!("void imported_webrogue_gfx_poll(uint32_t* out_len);");
        write!("");
        write!("__attribute__((import_name(\"poll_read\")))");
        write!("__attribute__((import_module(\"webrogue_gfx\")))");
        write!("void imported_webrogue_gfx_poll_read(void *buf);");
        write!("");
        write!("#define BUF_SIZE(LEN) if(available < LEN) {{\\");
        writer.inc_indent();
        {
            write!("buffer_consumed = buffer_used_size;\\");
            write!("result.type = WEBROGUE_EVENT_TYPE_INVALID;\\");
            write!("return result;\\");
        }
        writer.dec_indent();
        write!("}} buffer_consumed += LEN;");
        write!("#define RETURN return result;");
        write!("#define GET(TYPE, OFFSET) *((TYPE*)(current_pointer + OFFSET));");
        write!("");
        write!("webrogue_event webrogue_gfx_poll() {{");
        writer.inc_indent();
        {
            write!("webrogue_event result;");
            write!("static void* buffer_data = NULL;");
            write!("if(!buffer_data) {{");
            writer.inc_indent();
            {
                write!("buffer_data = malloc(WEBROGUE_MAX_ENCODED_EVENT_SIZE);");
            }
            writer.dec_indent();
            write!("}}");
            write!("static uint32_t buffer_max_size = WEBROGUE_MAX_ENCODED_EVENT_SIZE;");
            write!("static uint32_t buffer_used_size = 0;");
            write!("static uint32_t buffer_consumed = 0;");
            write!("uint32_t available = buffer_used_size - buffer_consumed;");
            write!("if(available == 0) {{");
            writer.inc_indent();
            {
                write!("uint32_t new_size;");
                write!("imported_webrogue_gfx_poll(&new_size);");
                write!("if(new_size > buffer_max_size) {{");
                writer.inc_indent();
                {
                    write!("free(buffer_data);");
                    write!("buffer_data = malloc(new_size);");
                    write!("buffer_max_size = new_size;");
                }
                writer.dec_indent();
                write!("}}");
                write!("if(new_size) {{");
                writer.inc_indent();
                {
                    write!("imported_webrogue_gfx_poll_read(buffer_data);");
                }
                writer.dec_indent();
                write!("}}");
                write!("buffer_used_size = new_size;");
                write!("buffer_consumed = 0;");
                write!("available = new_size;");
            }
            writer.dec_indent();
            write!("}}");
            write!("if(available < 4) {{");
            writer.inc_indent();
            {
                write!("buffer_consumed = buffer_used_size;");
                write!("result.type = WEBROGUE_EVENT_TYPE_INVALID;");
                write!("return result;");
            }
            writer.dec_indent();
            write!("}}");
            write!("const char* current_pointer = ((const char*)buffer_data) + buffer_consumed;");
            write!("result.type = GET(uint32_t, 0);");
            write!("switch (result.type) {{");
            writer.inc_indent();
            {
                for event in events::all() {
                    write!("case WEBROGUE_EVENT_TYPE_{}: {{", event.name.to_uppercase());
                    writer.inc_indent();
                    {
                        write!("BUF_SIZE({});", event.size);
                        for field in event.fields.clone() {
                            write!(
                                "result.inner.{}.{} = GET({}, {});",
                                event.name,
                                field.name,
                                field.ty.c_name(),
                                field.offset
                            );
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
                    write!("result.type = WEBROGUE_EVENT_TYPE_INVALID;");
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

    enc_writer.write_to_file(&args.enc_path)?;
    dec_h_writer.write_to_file(&args.dec_h_path)?;
    dec_c_writer.write_to_file(&args.dec_c_path)?;

    anyhow::Ok(())
}
