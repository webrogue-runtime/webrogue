use crate::types::*;

pub fn get_as_str(parse_results: &ParseResults) -> String {
    let mut result = "".to_owned();
    //     let mut result = "

    // extern \"C\" {\n"
    //         .to_owned();
    //     let mut keywords = std::collections::BTreeSet::new();
    //     keywords.insert("type");
    //     keywords.insert("ref");

    //     for command in parse_results.commands.clone() {
    //         result += &format!(
    //             "    pub fn {}({}) -> {};\n",
    //             command.name,
    //             command
    //                 .params
    //                 .iter()
    //                 .map(|param| {
    //                     let mut mapped_name = param.name.clone();
    //                     if keywords.contains(mapped_name.as_str()) {
    //                         mapped_name = format!("_{}", mapped_name)
    //                     }
    //                     format!("{}: {}", mapped_name, param.ty.to_rust_type())
    //                 })
    //                 .collect::<Vec<_>>()
    //                 .join(", "),
    //             command.ret.to_rust_type()
    //         );
    //     }
    //     result += "}\n";
    let mut seen_enums = std::collections::BTreeSet::new();
    for enum_case in parse_results.enums.clone() {
        if seen_enums.contains(&enum_case.name) {
            result += "// DUPLICATE ";
        }
        result += &format!(
            "pub const {}: {} = {};\n",
            enum_case.name, enum_case.ty, enum_case.value
        );
        seen_enums.insert(enum_case.name);
    }

    for (group_name, enum_cases) in parse_results.enum_groups.clone() {
        if group_name == "SpecialNumbers" {
            continue;
        }
        let group_name = "GLEnumGroup".to_owned() + &group_name;
        result += "#[derive(Clone, Copy)]\n";
        result += &format!("pub enum {} {{\n", group_name);
        for enum_case in enum_cases.clone() {
            result += &format!("    {},\n", enum_case.name);
        }
        result += "}\n";
        result += &format!("impl {} {{\n", group_name);
        result += "    #[rustfmt::skip]\n";
        result += &format!(
            "    pub fn from_raw(raw: u32) -> Option<{}> {{\n",
            group_name
        );
        result += "        match raw {\n";
        for enum_case in enum_cases.clone() {
            result += &format!(
                "            {} => Some({}::{}),\n",
                enum_case.value, group_name, enum_case.name
            );
        }
        result += "            _ => None,\n";
        result += "        }\n";
        result += "    }\n";
        result += "}\n";
    }

    result += &format!(
        "pub const EXTENSIONS: [&str; {}] = [\n",
        parse_results.extensions.len()
    );
    for extension in parse_results.extensions.clone() {
        result += &format!("    \"{}\",\n", extension);
    }
    result += "];\n";
    result
}
