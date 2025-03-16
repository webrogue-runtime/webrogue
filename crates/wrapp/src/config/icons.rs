#[derive(serde::Serialize, serde::Deserialize, Clone, PartialEq)]
pub struct Icons {
    pub normal: NormalIcon,
}

impl Icons {
    pub fn strip(self) -> Self {
        Self {
            normal: self.normal.strip(),
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone, PartialEq)]
pub struct NormalIcon {
    pub path: Option<String>,
    pub inset: f32,
    pub background: Background,
}
impl NormalIcon {
    pub fn strip(self) -> Self {
        Self {
            path: None,
            inset: self.inset,
            background: self.background.strip(),
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone, PartialEq)]
pub struct Background {
    pub red: f32,
    pub green: f32,
    pub blue: f32,
}
impl Background {
    pub fn strip(self) -> Self {
        self
    }
}
