use clap::Args;

#[derive(Args, Debug, Clone)]
pub struct LicensesCommand {}

impl LicensesCommand {
    pub fn run(&self) -> anyhow::Result<()> {
        for (name, license) in webrogue_licenses::get_licenses() {
            println!("\n=== {} ===\n\n{}", name, license);
        }
        println!("\n===\n\nIf information about licenses of some dependencies is missing or inaccurate, please open an issue or a PR to add it at https://github.com/webrogue-runtime/webrogue.");
        Ok(())
    }
}
