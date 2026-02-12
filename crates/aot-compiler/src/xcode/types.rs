use clap::ValueEnum;
use webrogue_wrapp::config::Config;

#[derive(Clone, Copy, Debug)]
pub enum Configuration {
    Debug,
    Release,
    ReleaseLocal,
}
impl ValueEnum for Configuration {
    fn value_variants<'a>() -> &'a [Self] {
        &[Self::Debug, Self::Release, Self::ReleaseLocal]
    }

    fn to_possible_value(&self) -> Option<clap::builder::PossibleValue> {
        match self {
            Configuration::Debug => Some(clap::builder::PossibleValue::new("debug")),
            Configuration::Release => Some(clap::builder::PossibleValue::new("release")),
            Configuration::ReleaseLocal => Some(clap::builder::PossibleValue::new("local")),
        }
    }
}

#[derive(Clone, Copy)]
pub enum Destination {
    MacOS,
    Ios,
    IOSSim,
}

#[derive(serde::Serialize, serde::Deserialize, PartialEq)]
pub struct Stamp {
    pub template_id: Vec<u8>,
    pub icons: webrogue_icons::IconsData,
    pub config: Config,
}
