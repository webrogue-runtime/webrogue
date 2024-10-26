use crate::types::*;

fn map_param_name(original: String) -> String {
    match original.as_str() {
        "type" => "_type".to_owned(),
        "ref" => "_ref".to_owned(),
        _ => original,
    }
}

pub fn get_as_str(parse_results: &ParseResults) -> String {
    let mut result = "#[rustfmt::skip]\npub struct ProcAddresses {\n".to_owned();
    let mut init_func_ptrs = r#"
#[rustfmt::skip]
impl ProcAddresses {
    #[allow(invalid_value)]
    pub fn new() -> Self {
        unsafe { std::mem::MaybeUninit::<ProcAddresses>::zeroed().assume_init() }
    }

    pub fn fill(&mut self, _context: &mut webrogue_gfx::Context) {
        unsafe {
"#
    .to_owned();
    for command in parse_results.commands.clone() {
        let mut ffi_arg_types = vec![];

        for param in command.params.clone() {
            let mapped_name = map_param_name(param.name.clone());

            ffi_arg_types.push(format!("{}: {}", mapped_name, param.ty.to_rust_type()));
        }
        result += &format!(
            "    pub {}: unsafe extern \"stdcall\" fn({}) -> {},\n",
            command.name,
            ffi_arg_types.join(","),
            match command.ret {
                GLType::Void => "()".to_owned(),
                _ => command.ret.to_rust_type(),
            },
        );
        init_func_ptrs += &format!(
            "            self.{} = std::mem::transmute(crate::utils::get_proc_address(_context, \"{}\"));\n",
            command.name, command.name
        );
    }
    result += "}\n";
    result += &init_func_ptrs;
    result += "        }\n";
    result += "    }\n";
    result += "}\n";
    result
}
