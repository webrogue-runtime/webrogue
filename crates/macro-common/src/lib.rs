#[derive(Clone, Copy)]
pub enum ValueType {
    U32,
    U64,
    I32,
    I64,
    F32,
    F64,
}

impl ValueType {
    pub fn is_64bit(&self) -> bool {
        self.type_infos().0
    }

    pub fn is_unsigned(&self) -> bool {
        self.type_infos().1 .0
    }

    pub fn is_signed(&self) -> bool {
        self.type_infos().1 .1
    }

    pub fn is_float(&self) -> bool {
        self.type_infos().1 .2
    }

    pub fn rust_type_str(&self) -> &str {
        match self {
            ValueType::U32 => "u32",
            ValueType::U64 => "u64",
            ValueType::I32 => "i32",
            ValueType::I64 => "i64",
            ValueType::F32 => "f32",
            ValueType::F64 => "f64",
        }
    }

    pub fn iterator() -> std::slice::Iter<'static, ValueType> {
        static DIRECTIONS: [ValueType; 6] = [
            ValueType::U32,
            ValueType::U64,
            ValueType::I32,
            ValueType::I64,
            ValueType::F32,
            ValueType::F64,
        ];
        DIRECTIONS.iter()
    }

    pub fn from_rust_type_str(rust_type_str: &str) -> Option<Self> {
        ValueType::iterator()
            .filter(|ty| ty.rust_type_str() == rust_type_str)
            .next()
            .cloned()
    }

    fn type_infos(&self) -> (bool, (bool, bool, bool)) {
        match self {
            ValueType::U32 => (false, (true, false, false)),
            ValueType::U64 => (true, (true, false, false)),
            ValueType::I32 => (false, (false, true, false)),
            ValueType::I64 => (true, (false, true, false)),
            ValueType::F32 => (false, (false, false, true)),
            ValueType::F64 => (true, (false, false, true)),
        }
    }
}

#[derive(Clone)]
pub struct Import {
    pub func_name: String,
    pub args: Vec<ValueType>,
    pub ret: Option<ValueType>,
}

pub fn parse_file(file_content: std::borrow::Cow<str>, result: &mut Vec<Import>) {
    for line in file_content.split("\n") {
        if line.is_empty() {
            continue;
        }
        let mut line = line.to_owned();
        // let colon_pos = line.find(":").unwrap();
        let bracket_pos = line.find("(").unwrap();
        let func_name = line[..bracket_pos].to_string();
        line = line[bracket_pos + 1..].to_string();
        let bracket_pos = line.find(")").unwrap();
        let args_str = line[..bracket_pos].to_string();
        line = line[bracket_pos + 6..].to_string();
        let bracket_pos = line.find(")").unwrap();
        let ret_str = line[..bracket_pos].to_string();

        let mut args: Vec<String> = vec![];

        for arg_str in args_str.split(", ") {
            if !arg_str.is_empty() {
                args.push(arg_str.to_string());
            }
        }

        result.push(Import {
            func_name: func_name,
            args: args
                .iter()
                .map(|arg| ValueType::from_rust_type_str(&arg).expect("Unknown type"))
                .collect(),
            ret: match ret_str.as_str() {
                "" => None,
                _ => Some(ValueType::from_rust_type_str(&ret_str).expect("Unknown type")),
            },
        })
    }
}

mod kw {
    syn::custom_keyword!(module);
    syn::custom_keyword!(defs);
    syn::custom_keyword!(public);
    syn::custom_keyword!(attribute);
}

struct Config {
    pub is_public: bool,
    pub modules: Vec<(String, ModuleConfig)>,
}

pub struct ImportedModule {
    pub attributes: Vec<String>,
    pub module_name: String,
    pub rust_module: String,
    pub funcs: Vec<Import>,
}

pub struct Imports {
    pub is_public: bool,
    pub modules: Vec<ImportedModule>,
}

impl syn::parse::Parse for Imports {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let contents;
        let _lbrace = syn::braced!(contents in input);
        let fields: syn::punctuated::Punctuated<ConfigField, syn::Token![,]> =
            contents.parse_terminated(ConfigField::parse, syn::Token![,])?;
        let config = Config::build(fields.into_iter(), input.span())?;
        let modules: syn::Result<Vec<ImportedModule>> = config
            .modules
            .iter()
            .map(|module_config| {
                let rust_module = module_config.1.module.clone();
                let wasm_module = module_config.0.clone();
                let file_content = match module_config.1.defs.clone() {
                    None => match wasm_module.as_str() {
                        // "wasi_snapshot_preview1" => {
                        //     String::from_utf8_lossy(include_bytes!("../../wasi/defs.in"))
                        // }
                        "webrogue_gl" => {
                            String::from_utf8_lossy(include_bytes!("../../gl/defs.in"))
                        }
                        "webrogue_gfx" => {
                            String::from_utf8_lossy(include_bytes!("../../gfx/defs.in"))
                        }
                        _ => {
                            return Err(syn::Error::new(
                                _lbrace.span.span(),
                                format!(
                                    "`defs` for \"{}\" is not specified, and no default one found",
                                    module_config.0
                                ),
                            ))
                        }
                    },
                    Some(def) => {
                        let path = std::path::Path::new(&def);
                        let path = path.canonicalize().map_err(|err| {
                            syn::Error::new(
                                _lbrace.span.span(),
                                format!("{}: {}", err.to_string(), path.display()),
                            )
                        })?;
                        let content = std::fs::read_to_string(path.clone()).map_err(|err| {
                            syn::Error::new(
                                _lbrace.span.span(),
                                format!("{}: {}", err.to_string(), path.display()),
                            )
                        })?;
                        std::borrow::Cow::from(content)
                    }
                };
                let mut imports: Vec<Import> = vec![];
                parse_file(file_content, &mut imports);
                Ok(ImportedModule {
                    attributes: module_config.1.attributes.clone(),
                    module_name: wasm_module,
                    rust_module: rust_module,
                    funcs: imports,
                })
            })
            .collect();
        Ok(Imports {
            is_public: config.is_public,
            modules: modules?,
        })
    }
}

impl Config {
    pub fn build(
        fields: impl Iterator<Item = ConfigField>,
        err_loc: proc_macro2::Span,
    ) -> syn::Result<Self> {
        let mut is_public = None;
        let mut modules = vec![];
        for f in fields {
            match f {
                ConfigField::Public(c) => {
                    if is_public.is_some() {
                        return Err(syn::Error::new(err_loc, "duplicate `public` field"));
                    }
                    is_public = Some(c);
                }
                ConfigField::Module(name, module_config) => modules.push((name, module_config)),
            }
        }
        Ok(Config {
            is_public: is_public.unwrap_or(false),
            modules,
        })
    }
}

enum ConfigField {
    Public(bool),
    Module(String, ModuleConfig),
}

impl syn::parse::Parse for ConfigField {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(kw::public) {
            input.parse::<kw::public>()?;
            input.parse::<syn::Token![:]>()?;
            Ok(ConfigField::Public(input.parse::<syn::LitBool>()?.value))
        } else if lookahead.peek(syn::LitStr) {
            let s = input.parse::<syn::LitStr>()?;
            input.parse::<syn::Token![:]>()?;
            Ok(ConfigField::Module(s.value(), input.parse()?))
        } else {
            Err(lookahead.error())
        }
    }
}

struct ModuleConfig {
    pub attributes: Vec<String>,
    pub defs: Option<String>,
    pub module: String,
}

impl syn::parse::Parse for ModuleConfig {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let contents;
        let _lbrace = syn::braced!(contents in input);
        let fields: syn::punctuated::Punctuated<ModuleConfigField, syn::Token![,]> =
            contents.parse_terminated(ModuleConfigField::parse, syn::Token![,])?;
        Ok(ModuleConfig::build(fields.into_iter(), input.span())?)
    }
}

enum ModuleConfigField {
    Attribute(String),
    Module(String),
    Defs(String),
}

impl syn::parse::Parse for ModuleConfigField {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(kw::attribute) {
            input.parse::<kw::attribute>()?;
            input.parse::<syn::Token![:]>()?;
            Ok(ModuleConfigField::Attribute(
                input.parse::<syn::LitStr>()?.value(),
            ))
        } else if lookahead.peek(kw::defs) {
            input.parse::<kw::defs>()?;
            input.parse::<syn::Token![:]>()?;
            Ok(ModuleConfigField::Defs(
                input.parse::<syn::LitStr>()?.value(),
            ))
        } else if lookahead.peek(kw::module) {
            input.parse::<kw::module>()?;
            input.parse::<syn::Token![:]>()?;
            let module_paths: syn::punctuated::Punctuated<syn::PathSegment, syn::Token![::]> =
                input.parse_terminated(syn::PathSegment::parse, syn::Token![::])?;
            let module_paths = module_paths
                .into_iter()
                .map(|segment| segment.ident.to_string());
            let module_path = module_paths.collect::<Vec<_>>().join("::");
            Ok(ModuleConfigField::Module(module_path))
        } else {
            Err(lookahead.error())
        }
    }
}

impl ModuleConfig {
    pub fn build(
        fields: impl Iterator<Item = ModuleConfigField>,
        err_loc: proc_macro2::Span,
    ) -> syn::Result<Self> {
        let mut defs = None;
        let mut module = None;
        let mut attributes = vec![];
        for f in fields {
            match f {
                ModuleConfigField::Attribute(attribute) => attributes.push(attribute),
                ModuleConfigField::Defs(path) => {
                    if defs.is_some() {
                        return Err(syn::Error::new(err_loc, "duplicate `defs` field"));
                    }
                    defs = Some(path);
                }
                ModuleConfigField::Module(name) => {
                    if module.is_some() {
                        return Err(syn::Error::new(err_loc, "duplicate `module` field"));
                    }
                    module = Some(name);
                }
            }
        }
        Ok(ModuleConfig {
            attributes,
            defs,
            module: module.ok_or_else(|| syn::Error::new(err_loc, "`module` field required"))?,
        })
    }
}

pub use syn::parse_macro_input;
use syn::spanned::Spanned;

pub fn make_context_fn(imports: &Imports) -> String {
    format!(
        "
{}fn make_context_vec(
    {}
) -> webrogue_runtime::ContextVec {{
    let mut result = webrogue_runtime::ContextVec::new();
    {}
    result
}}
    ",
        if imports.is_public { "pub " } else { "" },
        imports
            .modules
            .iter()
            .map(|imported_module| {
                format!(
                    "{}\n{}_ctx: *mut {}::Context,\n",
                    imported_module.attributes.join("\n"),
                    imported_module.module_name,
                    imported_module.rust_module
                )
            })
            .collect::<Vec<_>>()
            .join(""),
        imports
            .modules
            .iter()
            .enumerate()
            .map(|(i, imported_module)| {
                format!(
                    "{}\nresult.set({}, {}_ctx);\n",
                    imported_module.attributes.join("\n"),
                    i,
                    imported_module.module_name
                )
            })
            .collect::<Vec<_>>()
            .join("")
    )
}
