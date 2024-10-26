use crate::{common, types::*};

fn parse_requirements(feature_node: roxmltree::Node) -> FeatureRequirements {
    let mut result = FeatureRequirements::new();
    for require_node in feature_node.children() {
        if require_node.tag_name().name() != "require" {
            continue;
        }
        for node in require_node.children() {
            match node.tag_name().name() {
                "command" => result
                    .commands
                    .push(node.attribute("name").unwrap().to_owned()),
                "enum" => result
                    .enums
                    .push(node.attribute("name").unwrap().to_owned()),
                _ => {}
            }
        }
    }
    return result;
}

fn parse_extensions(extensions_node: roxmltree::Node) -> FeatureRequirements {
    let mut result = FeatureRequirements::new();
    for extension_node in extensions_node.children() {
        let name = match extension_node.attribute("name") {
            None => continue,
            Some(s) => s,
        };
        let supported_attribute = match extension_node.attribute("supported") {
            None => continue,
            Some(s) => s,
        };
        if !supported_attribute.split("|").any(|api| api == "gles2") {
            continue;
        }
        let mut current_requirements = FeatureRequirements::new();
        current_requirements.extensions.push(name.to_owned());
        for require_node in extension_node.children() {
            if require_node.tag_name().name() != "require" {
                continue;
            }
            for node in require_node.children() {
                match node.tag_name().name() {
                    "command" => current_requirements
                        .commands
                        .push(node.attribute("name").unwrap().to_owned()),
                    "enum" => current_requirements
                        .enums
                        .push(node.attribute("name").unwrap().to_owned()),
                    _ => {}
                }
            }
        }
        if current_requirements.commands.is_empty() || common::EXTRA_EXTENSIONS.contains(&name) {
            result.append(current_requirements);
        }
    }
    return result;
}

fn parse_type(proto_param_node: roxmltree::Node) -> Param {
    let mut ret_type = None;
    let mut is_const = false;
    let mut name = None;
    for proto_child in proto_param_node.children() {
        let text = proto_child.text().unwrap();
        let tag_name = proto_child.tag_name().name();
        match (tag_name, text) {
            ("name", v) => name = Some(v.to_owned()),
            ("", " ") => {}
            ("", "void ") => {
                ret_type = Some(GLType::Void);
            }
            ("", "const ") => is_const = true,
            ("ptype", ty) => {
                ret_type = Some(match ty {
                    "GLboolean" => GLType::U8,
                    "GLuint" => GLType::UInt,
                    "GLenum" => GLType::UInt,
                    "GLint" => GLType::Int,
                    "GLubyte" => GLType::U8,
                    "GLchar" => GLType::I8,
                    "GLfloat" => GLType::Float,
                    "GLsizeiptr" => GLType::ISizeT,
                    "GLintptr" => GLType::ISizeT,
                    "GLbitfield" => GLType::UInt,
                    "GLsizei" => GLType::Int,
                    "GLsync" => GLType::OpaqueSync,
                    "GLuint64" => GLType::U64,
                    "GLint64" => GLType::I64,
                    _ => panic!("{}", ty),
                });
            }
            ("", " *") => ret_type = Some(GLType::Ptr(Box::new(ret_type.unwrap()), is_const)),
            ("", "const void *") => ret_type = Some(GLType::Ptr(Box::new(GLType::Void), true)),
            ("", "void **") => {
                ret_type = Some(GLType::Ptr(
                    Box::new(GLType::Ptr(Box::new(GLType::Void), false)),
                    false,
                ))
            }
            ("", "void *") => ret_type = Some(GLType::Ptr(Box::new(GLType::Void), false)),
            ("", " *const*") => {
                ret_type = Some(GLType::Ptr(
                    Box::new(GLType::Ptr(Box::new(ret_type.unwrap()), is_const)),
                    true,
                ))
            }
            _ => panic!("({}, {})", tag_name, text),
        };
    }
    let len_name = proto_param_node
        .attribute("len")
        .and_then(|s| Some(s.to_owned()));
    let group = proto_param_node
        .attribute("group")
        .and_then(|s| Some(s.to_owned()));
    Param {
        name: name.unwrap(),
        ty: ret_type.unwrap(),
        len_name: len_name,
        group: group,
    }
}

fn parse_commands(
    commands_node: roxmltree::Node,
    requirements: &FeatureRequirements,
) -> Vec<Command> {
    let mut commands = vec![];
    for command_node in commands_node.children() {
        if command_node.children().count() == 0 {
            continue;
        }
        let proto_node = command_node
            .children()
            .find(|node| node.tag_name().name() == "proto")
            .unwrap();
        let name_node = proto_node
            .children()
            .find(|node| node.tag_name().name() == "name")
            .unwrap();
        let name = name_node.text().unwrap();
        if requirements
            .commands
            .iter()
            .find(|command_name| command_name.as_str() == name)
            .is_none()
        {
            continue;
        }
        let mut params = vec![];
        let ret_type = parse_type(proto_node).ty;
        for param_node in command_node
            .children()
            .filter(|node| node.tag_name().name() == "param")
        {
            let param = parse_type(param_node);
            params.push(param);
        }
        commands.push(Command {
            name: name.to_owned(),
            ret: ret_type,
            params: params,
        });
    }
    return commands;
}

// TODO parse enum groups
fn parse_enums(
    enums_node: roxmltree::Node,
    requirements: &FeatureRequirements,
    enums: &mut Vec<EnumCase>,
    enum_groups: &mut std::collections::BTreeMap<String, Vec<EnumCase>>,
) {
    for enum_node in enums_node.children() {
        if enum_node.tag_name().name() != "enum" {
            continue;
        }
        let name = enum_node.attribute("name").unwrap().to_owned();
        if !requirements.enums.contains(&name) {
            continue;
        }
        let value = enum_node.attribute("value").unwrap();
        // let parsed_value = u32::from_str_radix(&value[2..], 16).unwrap();
        let mut ty = match enum_node.attribute("type") {
            None => "u32",
            Some("ull") => "u64",
            Some("u") => "u32",
            v => {
                dbg!(v);
                unimplemented!();
            }
        };
        if value.starts_with("-") && enum_node.attribute("type").is_none() {
            ty = "i32";
        }
        let enum_case = EnumCase {
            name: name,
            value: value.to_owned(),
            ty: ty.to_owned(),
        };
        if let Some(groups) = enum_node.attribute("group") {
            for group in groups.split(',') {
                let group = group.to_owned();
                match enum_groups.get_mut(&group) {
                    None => {
                        enum_groups.insert(group, vec![enum_case.clone()]);
                    }
                    Some(cases) => cases.push(enum_case.clone()),
                };
            }
        }
        enums.push(enum_case);
    }
}

pub fn parse() -> ParseResults {
    let xml_str = include_str!("../gl.xml");
    let doc = roxmltree::Document::parse(&xml_str).unwrap();
    let mut requirements = FeatureRequirements::new();
    {
        let feature_node = doc
            .descendants()
            .find(|node| {
                node.tag_name().name() == "feature"
                    && node.attribute("api").unwrap() == "gles2"
                    && node.attribute("number").unwrap() == "2.0"
            })
            .unwrap();
        requirements.append(parse_requirements(feature_node));
    }
    {
        let feature_node = doc
            .descendants()
            .find(|node| {
                node.tag_name().name() == "feature"
                    && node.attribute("api").unwrap() == "gles2"
                    && node.attribute("number").unwrap() == "3.0"
            })
            .unwrap();
        requirements.append(parse_requirements(feature_node));
    }
    let extensions_node = doc
        .descendants()
        .find(|node| node.tag_name().name() == "extensions")
        .unwrap();
    requirements.append(parse_extensions(extensions_node));

    let command_node = doc
        .descendants()
        .find(|node| node.tag_name().name() == "commands")
        .unwrap();
    let commands = parse_commands(command_node, &requirements);
    let mut enums = vec![];
    let mut enum_groups = std::collections::BTreeMap::new();
    let registry_node = doc.root_element();
    for node in registry_node.children() {
        match node.tag_name().name() {
            "enums" => parse_enums(node, &requirements, &mut enums, &mut enum_groups),
            _ => {}
        }
    }
    ParseResults {
        commands,
        enums,
        enum_groups,
        extensions: requirements.extensions,
    }
}
