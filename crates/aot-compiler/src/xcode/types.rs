#[derive(Clone, Copy)]
pub enum Configuration {
    Debug,
}

#[derive(Clone, Copy)]
pub enum Destination {
    MacOS,
    IOS,
    IOSSim,
}

#[derive(serde::Serialize, serde::Deserialize, PartialEq)]
pub struct Stamp {
    pub icons: IconsStamp,
}
#[derive(serde::Serialize, serde::Deserialize, PartialEq)]
pub struct IconsStamp {
    pub config: webrogue_wrapp::config::icons::Icons,
    pub normal_icon_bytes: Option<Vec<u8>>,
}
