use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub struct Icons {
    #[schemars(title = "Normal icon configuration")]
    pub normal: NormalIcon,
}

impl Icons {
    pub fn strip(self) -> Self {
        Self {
            normal: self.normal.strip(),
        }
    }
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub struct NormalIcon {
    #[schemars(
        title = "Relative path to application icon",
        description = "Despite being marked as not required, ommiting this field may cause unexpected errors buring builds mentioning missing 'normal_icon' or something like that"
    )]
    pub path: Option<String>,
    #[schemars(
        title = "Icon's relative inset",
        description = "It is relative to size of the whole icon, so it should be in range of [0..0.5)"
    )]
    pub inset: f32,
    #[schemars(
        title = "Icon's background color",
        description = "This color will be used to fill insets and transparent parts of your icon"
    )]
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

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub struct Background {
    #[schemars(
        title = "Red component of color",
        description = "Should be in range of [0..1]"
    )]
    pub red: f32,
    #[schemars(
        title = "Green component of color",
        description = "Should be in range of [0..1]"
    )]
    pub green: f32,
    #[schemars(
        title = "Blue component of color",
        description = "Should be in range of [0..1]"
    )]
    pub blue: f32,
}
impl Background {
    pub fn strip(self) -> Self {
        self
    }
}
