#[derive(serde::Serialize, serde::Deserialize, PartialEq)]
pub struct Stamp {
    pub template_id: Vec<u8>,
    pub icons: IconsStamp,
}
#[derive(serde::Serialize, serde::Deserialize, PartialEq)]
pub struct IconsStamp {
    pub config: webrogue_wrapp::config::icons::Icons,
    pub normal_icon_bytes: Option<Vec<u8>>,
}
