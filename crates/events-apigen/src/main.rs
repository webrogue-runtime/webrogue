mod code_writer;

#[derive(clap::Parser, Clone)]
struct Cli {
    c_header_path: std::path::PathBuf,
    c_source_path: std::path::PathBuf,
    wit_path: std::path::PathBuf,
}

fn main() -> anyhow::Result<()> {
    let args = <Cli as clap::Parser>::parse();

    let mut c_header_writer = code_writer::CodeWriter::new(args.c_header_path).unwrap();
    let mut c_source_writer = code_writer::CodeWriter::new(args.c_source_path).unwrap();
    let mut wit_writer = code_writer::CodeWriter::new(args.wit_path).unwrap();

    let mut writer = &mut c_header_writer;
    let mut indent = writer.indent_storage();
    macro_rules! w {
        ($($arg:tt)*) => {
            writer.writeln(&format!($($arg)*)).unwrap()
        };
    }
    {
        w!("// Events");
        for r#enum in webrogue_events::enums() {
            w!("typedef enum {} {{", r#enum.c_name_t());
            indent.inc(|| {
                for case in r#enum.cases.clone() {
                    w!("{} = {},", case.c_name(&r#enum), case.value);
                }
                w!("");
                w!(
                    "{}_MAX = {},",
                    r#enum.c_name().to_uppercase(),
                    r#enum.ty.c_max()
                );
            });
            w!("}} {};", r#enum.c_name_t());
            w!("");
        }
        for event in webrogue_events::events() {
            if event.fields.is_empty() {
                continue;
            }
            w!("struct {} {{", event.c_struct_name());
            indent.inc(|| {
                for field in event.fields.clone() {
                    let array_len = match field.ty {
                        webrogue_events::FieldType::Bytes(len) => {
                            w!("size_t {}_len;", field.c_name());
                            format!("[{}]", len)
                        }
                        _ => "".to_owned(),
                    };
                    w!("{} {}{};", field.ty.c_name(), field.c_name(), array_len);
                }
            });
            w!("}};");
            w!("");
        }
        w!("enum wr4c_event_tag_t {{");
        indent.inc(|| {
            w!("WR4C_EVENT_TAG_INVALID = 0,");
            for event in webrogue_events::events() {
                w!("{} = {},", event.c_case_name(), event.id);
            }
        });
        w!("}};");
        w!("");
        w!("typedef struct wr4c_event_t {{");
        indent.inc(|| {
            w!("enum wr4c_event_tag_t tag;");
            w!("union {{");
            indent.inc(|| {
                for event in webrogue_events::events() {
                    if event.fields.is_empty() {
                        continue;
                    }
                    w!("struct {} {};", event.c_struct_name(), event.c_union_name());
                }
            });
            w!("}} inner;");
            w!("wr4c_window_t window;");
        });
        w!("}} wr4c_event_t;");
    }

    writer = &mut c_source_writer;
    indent = writer.indent_storage();
    {
        w!("");
        for r#enum in webrogue_events::enums() {
            w!(
                "static {} convert_{}({} data) {{",
                r#enum.c_name_t(),
                r#enum.c_name(),
                r#enum.c_bindgen_name()
            );
            indent.inc(|| {
                w!("switch (data) {{");
                indent.inc(|| {
                    let unknown_case = r#enum
                        .cases
                        .iter()
                        .find(|r#case| r#case.name == "unknown")
                        .expect("Should have \"unknown\" case")
                        .clone();
                    for r#case in r#enum.cases.clone() {
                        w!(
                            "case {}: return {};",
                            r#case.c_bindgen_name(&r#enum),
                            r#case.c_name(&r#enum)
                        );
                    }
                    w!("default: return {};", unknown_case.c_name(&r#enum));
                });
                w!("}}");
            });
            w!("}}");
        }
        w!("static wr4c_event_t convert_webrogue_event(webrogue_gfx_windowing_window_event_t event) {{");
        indent.inc(|| {
            w!("wr4c_event_t result;");
            w!("switch (event.tag) {{");
            indent.inc(|| {
                for event in webrogue_events::events() {
                    w!("case {}: {{", event.c_wit_name());
                    indent.inc(|| {
                        w!("result.tag = {};", event.c_case_name());
                        for (i, field) in event.fields.clone().into_iter().enumerate() {
                            let wit_tuple_field = format!("f{i}");
                            let field_c_name = field.c_name();
                            match field.ty {
                                webrogue_events::FieldType::Enum(r#enum) => w!(
                                    "result.inner.{}.{} = convert_{}(event.val.{}.{});",
                                    event.c_union_name(),
                                    field_c_name,
                                    r#enum.c_name(),
                                    event.c_union_name(),
                                    wit_tuple_field,
                                ),

                                webrogue_events::FieldType::Raw(_) => w!(
                                    "result.inner.{}.{} = event.val.{}.{};",
                                    event.c_union_name(),
                                    field_c_name,
                                    event.c_union_name(),
                                    wit_tuple_field,
                                ),
                                webrogue_events::FieldType::Bytes(len) => {
                                    w!(
                                        "memcpy(&result.inner.{}.{}, event.val.{}.{}.ptr, {});",
                                        event.c_union_name(),
                                        field_c_name,
                                        event.c_union_name(),
                                        wit_tuple_field,
                                        len,
                                    );
                                    w!(
                                        "result.inner.{}.{}_len = event.val.{}.{}.len;",
                                        event.c_union_name(),
                                        field_c_name,
                                        event.c_union_name(),
                                        wit_tuple_field,
                                    );
                                }
                            };
                        }
                        w!("return result;");
                    });
                    w!("}}");
                }
            });

            indent.inc(|| {
                w!("default: {{");
                indent.inc(|| {
                    w!("result.tag = WR4C_EVENT_TAG_INVALID;");
                    w!("return result;");
                });
                w!("}}");
            });
            w!("}}");
        });
        w!("}}");
    }

    writer = &mut wit_writer;
    indent = writer.indent_storage();
    {
        indent.inc(|| {
            for r#enum in webrogue_events::enums() {
                w!("enum {} {{", r#enum.wit_name());
                indent.inc(|| {
                    for case in r#enum.cases.clone() {
                        w!("{},", case.wit_name());
                    }
                });
                w!("}}");
                w!("");
            }

            w!("variant window-event {{");
            indent.inc(|| {
                for event in webrogue_events::events() {
                    let mut line = event.wit_name();
                    if !event.fields.is_empty() {
                        let mut comment = "/// ".to_string();
                        line += "(tuple<";
                        for (i, field) in event.fields.iter().enumerate() {
                            if i != 0 {
                                line += ", ";
                                comment += ", ";
                            }
                            line += &field.ty.wit_name();
                            comment += &field.wit_name();
                        }
                        line += ">)";
                        w!("{}", comment);
                    }
                    w!("{},", line);
                }
            });
            w!("}}");
        });
        writer.write_raw("    ")?;
    }

    c_header_writer.write_to_file()?;
    c_source_writer.write_to_file()?;
    wit_writer.write_to_file()?;

    Ok(())
}
