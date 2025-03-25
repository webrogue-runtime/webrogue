use syn::parse_macro_input;
mod link;
use quote::quote;

#[proc_macro]
pub fn wr_web_integration(args: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let config = parse_macro_input!(args as wiggle_generate::WasmtimeConfig);
    let doc = config.c.load_document();

    let settings = wiggle_generate::CodegenSettings::new(
        &config.c.errors,
        &config.c.async_,
        &doc,
        true,
        &config.c.tracing,
        config.c.mutable,
    )
    .expect("validating codegen settings");

    let modules = doc
        .modules()
        .map(|module| link::link_module(&module, Some(&config.target), &settings));
    quote!( #(#modules)* ).into()
}
