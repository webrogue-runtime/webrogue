use schemars::JsonSchema;
use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::config::icons::{ColoredIcon, IconBrightness};

pub mod icons;

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub struct Config {
    #[schemars(
        title = "Human-readable application name",
        description = "This parameter is also used as application name on Android, macOS and iOS"
    )]
    pub name: String,
    #[schemars(
        title = "Application identifier",
        description = "Apple-style application identifier. Same value will be used as bundle ID for macOS and iOS applications. Lowercased value will be used as Android Application ID."
    )]
    pub id: String,
    #[schemars(
        title = "Application entrypoint file path",
        description = "Relative path to WebAssembly module file. 'main.wasm' is assumed if this value is not specified."
    )]
    pub main: Option<String>,
    #[schemars(title = "Filesystem configuration")]
    pub filesystem: Option<FilesystemConfig>,
    #[schemars(
        title = "Icons's configuration",
        description = "This field is required to build for Android,  macOS and iOS."
    )]
    pub icons: Option<icons::Icons>,
    #[schemars(
        title = "Application version",
        description = "Application's semantic version. Read https://semver.org/ to learn about format of this value."
    )]
    pub version: semver::Version,
    #[schemars(
        title = "Environment variables",
        description = "These values are passed to your application"
    )]
    pub env: Option<HashMap<String, String>>,
    #[schemars(title = "Graphics configuration")]
    pub graphics_api: Option<GraphicsApiConfig>,
}
impl Config {
    pub fn strip(self) -> Self {
        Self {
            name: self.name,
            id: self.id,
            main: None,
            filesystem: self.filesystem.map(|filesystem| filesystem.strip()),
            icons: self.icons.map(|icons| icons.strip()),
            version: self.version,
            env: self.env,
            graphics_api: self.graphics_api,
        }
    }

    pub fn vulkan_requirement(&self) -> Requirement {
        self.graphics_api
            .as_ref()
            .and_then(|graphics_api| graphics_api.vulkan.clone())
            .unwrap_or(Requirement::Required)
    }

    pub fn light_icon(&self) -> Option<ColoredIcon> {
        self.icons.as_ref()?.light.clone()
    }

    pub fn dark_icon(&self) -> Option<ColoredIcon> {
        self.icons.as_ref()?.dark.clone()
    }

    pub fn default_icon_brightness(&self) -> IconBrightness {
        self.icons
            .as_ref()
            .and_then(|icons| icons.default_brightness.clone())
            .unwrap_or(IconBrightness::LIGHT)
    }
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub struct FilesystemConfig {
    #[schemars(
        title = "Resources configuration",
        description = "Resources (a.k.a Assets), are read-only files and directories packaged with your application"
    )]
    pub resources: Option<Vec<FilesystemResourceConfig>>,
    #[schemars(
        title = "Persistent storage configuration",
        description = "Persistent storages (a.k.a Volumes), are read-write directories where your application can store data. This data is persisted between runs. Directory is empty by default."
    )]
    pub persistent: Option<Vec<FilesystemPersistentConfig>>,
}
impl FilesystemConfig {
    pub fn strip(self) -> Self {
        Self {
            resources: None,
            persistent: self.persistent.map(|persistent| {
                persistent
                    .into_iter()
                    .map(|persistent| persistent.strip())
                    .collect()
            }),
        }
    }
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub struct FilesystemResourceConfig {
    #[schemars(
        title = "Path to resource",
        description = "This path must lead to file or directory relatively to directory where your configuration files is stored."
    )]
    pub real_path: String,
    #[schemars(
        title = "Mapped path to resource",
        description = "This is the absolute path where your application can find this resource"
    )]
    pub mapped_path: String,
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub struct FilesystemPersistentConfig {
    #[schemars(
        title = "Persistent storage name",
        description = "Persistent storages are identified by this name. Same name can be reused to mount the same persistent storage at different paths"
    )]
    pub name: String,
    #[schemars(
        title = "Mapped path to resource",
        description = "This is the absolute path where your application can find this persistent storage"
    )]
    pub mapped_path: String,
}
impl FilesystemPersistentConfig {
    pub fn strip(self) -> Self {
        self
    }
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub struct GraphicsApiConfig {
    #[schemars(
        title = "Enablement of CPU rendering API",
        description = "CPU rendering (a.k.a software rendering) is a way to just display an array of pixels stored directly in memory. This is the simplest and the most widely supported way to present something as it may bypass GPU entriely, but performance is expected to be low, so it is mostly used as a fallback if Vulkan is unsupported. This property actually does nothing, as CPU rendering is always available."
    )]
    pub cpu_rendering: Option<Requirement>,
    #[schemars(
        title = "Enablement of Vulkan API",
        description = "These value determines weather your applications is able to use Vulkan, and weather it deppends on it. Some platforms like Web browsers, old/virtualized Windows machines and iOS simulators may not support Vulkan, so your application will fail early if Vulkan is required but unsupported. On the other hand, disabling Vulkan entriely can strip away GFXStream library and save some size of final application"
    )]
    pub vulkan: Option<Requirement>,
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub enum Requirement {
    #[serde(rename = "disabled")]
    Disabled,
    #[serde(rename = "optional")]
    Optional,
    #[serde(rename = "required")]
    Required,
}

impl Requirement {
    pub fn to_bool_option(&self) -> Option<bool> {
        match self {
            Requirement::Disabled => Some(false),
            Requirement::Optional => None,
            Requirement::Required => Some(true),
        }
    }
}
