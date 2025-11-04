#[derive(serde::Serialize, serde::Deserialize, PartialEq)]
pub struct Stamp {
    pub template_id: Vec<u8>,
    pub icons: webrogue_icons::IconsData,
}
