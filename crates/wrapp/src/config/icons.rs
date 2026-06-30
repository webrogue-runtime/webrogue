use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const LIGHT_ICON_UNCOMPRESSED_NAME: &str = "light_icon";
pub const DARK_ICON_UNCOMPRESSED_NAME: &str = "dark_icon";

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub struct Icons {
    #[schemars(title = "Light icon configuration. Defaults to Webrogue logo.")]
    pub light: Option<ColoredIcon>,
    #[schemars(
        title = "Dark icon configuration. Defaults to light icon, or Webrogue logo if light icon is not specified."
    )]
    pub dark: Option<ColoredIcon>,
    #[schemars(
        title = "What icon Webrogue should default to it it's unable to generate separate light and dark icons."
    )]
    pub default_brightness: Option<IconBrightness>,
}

impl Icons {
    pub fn strip(self) -> Self {
        Self {
            light: self.light.map(ColoredIcon::strip),
            dark: self.dark.map(ColoredIcon::strip),
            default_brightness: self.default_brightness,
        }
    }
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub struct ColoredIcon {
    #[schemars(
        title = "Relative path to application icon",
        description = "Icons fall back to path from another brightness or Webrogue logo if this field is not specified."
    )]
    pub path: Option<String>,
    #[schemars(
        title = "Icon's relative inset",
        description = "It is relative to size of the whole icon. Should be in range of [0..1). Android guarantees that you icon is not clipped if inset is 0.28 or more"
    )]
    pub inset: f32,
    #[schemars(
        title = "Icon's background color",
        description = "This color will be used to fill insets and transparent parts of your icon. Specified as six-digit hexadecimal web color.",
        regex(pattern = r"^#?([a-fA-F0-9]{6})$")
    )]
    pub background: String,
}
impl ColoredIcon {
    pub fn strip(self) -> Self {
        Self {
            path: None,
            inset: self.inset,
            background: self.background,
        }
    }
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub enum IconBrightness {
    #[serde(rename = "light")]
    LIGHT,
    #[serde(rename = "dark")]
    DARK,
}
