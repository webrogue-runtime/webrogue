#![allow(unused_qualifications)]

use http::HeaderValue;
use validator::Validate;

#[cfg(feature = "server")]
use crate::header;
use crate::{models, types::*};

#[allow(dead_code)]
fn from_validation_error(e: validator::ValidationError) -> validator::ValidationErrors {
  let mut errs = validator::ValidationErrors::new();
  errs.add("na", e);
  errs
}

#[allow(dead_code)]
pub fn check_xss_string(v: &str) -> std::result::Result<(), validator::ValidationError> {
    if ammonia::is_html(v) {
        std::result::Result::Err(validator::ValidationError::new("xss detected"))
    } else {
        std::result::Result::Ok(())
    }
}

#[allow(dead_code)]
pub fn check_xss_vec_string(v: &[String]) -> std::result::Result<(), validator::ValidationError> {
    if v.iter().any(|i| ammonia::is_html(i)) {
        std::result::Result::Err(validator::ValidationError::new("xss detected"))
    } else {
        std::result::Result::Ok(())
    }
}

#[allow(dead_code)]
pub fn check_xss_map_string(
    v: &std::collections::HashMap<String, String>,
) -> std::result::Result<(), validator::ValidationError> {
    if v.keys().any(|k| ammonia::is_html(k)) || v.values().any(|v| ammonia::is_html(v)) {
        std::result::Result::Err(validator::ValidationError::new("xss detected"))
    } else {
        std::result::Result::Ok(())
    }
}

#[allow(dead_code)]
pub fn check_xss_map_nested<T>(
    v: &std::collections::HashMap<String, T>,
) -> std::result::Result<(), validator::ValidationError>
where
    T: validator::Validate,
{
    if v.keys().any(|k| ammonia::is_html(k)) || v.values().any(|v| v.validate().is_err()) {
        std::result::Result::Err(validator::ValidationError::new("xss detected"))
    } else {
        std::result::Result::Ok(())
    }
}

#[allow(dead_code)]
pub fn check_xss_map<T>(v: &std::collections::HashMap<String, T>) -> std::result::Result<(), validator::ValidationError> {
    if v.keys().any(|k| ammonia::is_html(k)) {
        std::result::Result::Err(validator::ValidationError::new("xss detected"))
    } else {
        std::result::Result::Ok(())
    }
}





/// RGB color specification
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct Color {
    /// Red component of color
    #[serde(rename = "red")]
    #[validate(
            range(min = 0f64, max = 1f64),
    )]
    pub red: f64,

    /// Green component of color
    #[serde(rename = "green")]
    #[validate(
            range(min = 0f64, max = 1f64),
    )]
    pub green: f64,

    /// Blue component of color
    #[serde(rename = "blue")]
    #[validate(
            range(min = 0f64, max = 1f64),
    )]
    pub blue: f64,

}



impl Color {
    #[allow(clippy::new_without_default, clippy::too_many_arguments)]
    pub fn new(red: f64, green: f64, blue: f64, ) -> Color {
        Color {
 red,
 green,
 blue,
        }
    }
}

/// Converts the Color value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params: Vec<Option<String>> = vec![

            Some("red".to_string()),
            Some(self.red.to_string()),


            Some("green".to_string()),
            Some(self.green.to_string()),


            Some("blue".to_string()),
            Some(self.blue.to_string()),

        ];

        write!(f, "{}", params.into_iter().flatten().collect::<Vec<_>>().join(","))
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a Color value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for Color {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub red: Vec<f64>,
            pub green: Vec<f64>,
            pub blue: Vec<f64>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing Color".to_string())
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "red" => intermediate_rep.red.push(<f64 as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "green" => intermediate_rep.green.push(<f64 as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "blue" => intermediate_rep.blue.push(<f64 as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    _ => return std::result::Result::Err("Unexpected key while parsing Color".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(Color {
            red: intermediate_rep.red.into_iter().next().ok_or_else(|| "red missing in Color".to_string())?,
            green: intermediate_rep.green.into_iter().next().ok_or_else(|| "green missing in Color".to_string())?,
            blue: intermediate_rep.blue.into_iter().next().ok_or_else(|| "blue missing in Color".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<Color> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<Color>> for HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<Color>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(format!(r#"Invalid header value for Color - value: {hdr_value} is invalid {e}"#))
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue> for header::IntoHeaderValue<Color> {
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <Color as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(format!(r#"Unable to convert header value '{value}' into Color - {err}"#))
                    }
             },
             std::result::Result::Err(e) => std::result::Result::Err(format!(r#"Unable to convert header: {hdr_value:?} to string: {e}"#))
        }
    }
}



/// Icon configuration
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct Icon {
    /// Relative path to application icon
    #[serde(rename = "path")]
          #[validate(custom(function = "check_xss_string"))]
    pub path: String,

    /// Icon's relative inset
    #[serde(rename = "inset")]
    #[validate(
            range(min = 0f64, max = 0.5f64),
    )]
    pub inset: f64,

    #[serde(rename = "background")]
          #[validate(nested)]
    pub background: models::Color,

}



impl Icon {
    #[allow(clippy::new_without_default, clippy::too_many_arguments)]
    pub fn new(path: String, inset: f64, background: models::Color, ) -> Icon {
        Icon {
 path,
 inset,
 background,
        }
    }
}

/// Converts the Icon value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for Icon {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params: Vec<Option<String>> = vec![

            Some("path".to_string()),
            Some(self.path.to_string()),


            Some("inset".to_string()),
            Some(self.inset.to_string()),

            // Skipping background in query parameter serialization

        ];

        write!(f, "{}", params.into_iter().flatten().collect::<Vec<_>>().join(","))
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a Icon value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for Icon {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub path: Vec<String>,
            pub inset: Vec<f64>,
            pub background: Vec<models::Color>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing Icon".to_string())
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "path" => intermediate_rep.path.push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "inset" => intermediate_rep.inset.push(<f64 as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "background" => intermediate_rep.background.push(<models::Color as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    _ => return std::result::Result::Err("Unexpected key while parsing Icon".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(Icon {
            path: intermediate_rep.path.into_iter().next().ok_or_else(|| "path missing in Icon".to_string())?,
            inset: intermediate_rep.inset.into_iter().next().ok_or_else(|| "inset missing in Icon".to_string())?,
            background: intermediate_rep.background.into_iter().next().ok_or_else(|| "background missing in Icon".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<Icon> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<Icon>> for HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<Icon>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(format!(r#"Invalid header value for Icon - value: {hdr_value} is invalid {e}"#))
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue> for header::IntoHeaderValue<Icon> {
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <Icon as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(format!(r#"Unable to convert header value '{value}' into Icon - {err}"#))
                    }
             },
             std::result::Result::Err(e) => std::result::Result::Err(format!(r#"Unable to convert header: {hdr_value:?} to string: {e}"#))
        }
    }
}



#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct Sdp {
    /// SDP string
    #[serde(rename = "sdp")]
          #[validate(custom(function = "check_xss_string"))]
    pub sdp: String,

}



impl Sdp {
    #[allow(clippy::new_without_default, clippy::too_many_arguments)]
    pub fn new(sdp: String, ) -> Sdp {
        Sdp {
 sdp,
        }
    }
}

/// Converts the Sdp value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for Sdp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params: Vec<Option<String>> = vec![

            Some("sdp".to_string()),
            Some(self.sdp.to_string()),

        ];

        write!(f, "{}", params.into_iter().flatten().collect::<Vec<_>>().join(","))
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a Sdp value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for Sdp {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub sdp: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing Sdp".to_string())
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "sdp" => intermediate_rep.sdp.push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    _ => return std::result::Result::Err("Unexpected key while parsing Sdp".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(Sdp {
            sdp: intermediate_rep.sdp.into_iter().next().ok_or_else(|| "sdp missing in Sdp".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<Sdp> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<Sdp>> for HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<Sdp>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(format!(r#"Invalid header value for Sdp - value: {hdr_value} is invalid {e}"#))
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue> for header::IntoHeaderValue<Sdp> {
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <Sdp as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(format!(r#"Unable to convert header value '{value}' into Sdp - {err}"#))
                    }
             },
             std::result::Result::Err(e) => std::result::Result::Err(format!(r#"Unable to convert header: {hdr_value:?} to string: {e}"#))
        }
    }
}



#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct WebrogueConfig {
    /// Human-readable application name
    #[serde(rename = "name")]
          #[validate(custom(function = "check_xss_string"))]
    pub name: String,

    /// Apple-style application identifier. Same value will be used as bundle ID for macOS and iOS applications. Lowercased value will be used as Android Application ID.
    #[serde(rename = "id")]
          #[validate(custom(function = "check_xss_string"))]
    pub id: String,

    /// Relative path to WebAssembly module file. 'main.wasm' is assumed if this value is not specified.
    #[serde(rename = "main")]
          #[validate(custom(function = "check_xss_string"))]
    #[serde(skip_serializing_if="Option::is_none")]
    pub main: Option<String>,

    /// Application semantic version. Read https://semver.org/ to learn about format of this value.
    #[serde(rename = "version")]
          #[validate(custom(function = "check_xss_string"))]
    pub version: String,

    #[serde(rename = "icons")]
          #[validate(nested)]
    #[serde(skip_serializing_if="Option::is_none")]
    pub icons: Option<models::WebrogueConfigIcons>,

    #[serde(rename = "filesystem")]
          #[validate(nested)]
    #[serde(skip_serializing_if="Option::is_none")]
    pub filesystem: Option<models::WebrogueConfigFilesystem>,

    /// Environment variables
    #[serde(rename = "env")]
          #[validate(custom(function = "check_xss_map_string"))]
    #[serde(skip_serializing_if="Option::is_none")]
    pub env: Option<std::collections::HashMap<String, String>>,

}



impl WebrogueConfig {
    #[allow(clippy::new_without_default, clippy::too_many_arguments)]
    pub fn new(name: String, id: String, version: String, ) -> WebrogueConfig {
        WebrogueConfig {
 name,
 id,
 main: None,
 version,
 icons: None,
 filesystem: None,
 env: None,
        }
    }
}

/// Converts the WebrogueConfig value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for WebrogueConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params: Vec<Option<String>> = vec![

            Some("name".to_string()),
            Some(self.name.to_string()),


            Some("id".to_string()),
            Some(self.id.to_string()),


            self.main.as_ref().map(|main| {
                [
                    "main".to_string(),
                    main.to_string(),
                ].join(",")
            }),


            Some("version".to_string()),
            Some(self.version.to_string()),

            // Skipping icons in query parameter serialization

            // Skipping filesystem in query parameter serialization

            // Skipping env in query parameter serialization

        ];

        write!(f, "{}", params.into_iter().flatten().collect::<Vec<_>>().join(","))
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a WebrogueConfig value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for WebrogueConfig {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub name: Vec<String>,
            pub id: Vec<String>,
            pub main: Vec<String>,
            pub version: Vec<String>,
            pub icons: Vec<models::WebrogueConfigIcons>,
            pub filesystem: Vec<models::WebrogueConfigFilesystem>,
            pub env: Vec<std::collections::HashMap<String, String>>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing WebrogueConfig".to_string())
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "name" => intermediate_rep.name.push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "id" => intermediate_rep.id.push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "main" => intermediate_rep.main.push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "version" => intermediate_rep.version.push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "icons" => intermediate_rep.icons.push(<models::WebrogueConfigIcons as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "filesystem" => intermediate_rep.filesystem.push(<models::WebrogueConfigFilesystem as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    "env" => return std::result::Result::Err("Parsing a container in this style is not supported in WebrogueConfig".to_string()),
                    _ => return std::result::Result::Err("Unexpected key while parsing WebrogueConfig".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(WebrogueConfig {
            name: intermediate_rep.name.into_iter().next().ok_or_else(|| "name missing in WebrogueConfig".to_string())?,
            id: intermediate_rep.id.into_iter().next().ok_or_else(|| "id missing in WebrogueConfig".to_string())?,
            main: intermediate_rep.main.into_iter().next(),
            version: intermediate_rep.version.into_iter().next().ok_or_else(|| "version missing in WebrogueConfig".to_string())?,
            icons: intermediate_rep.icons.into_iter().next(),
            filesystem: intermediate_rep.filesystem.into_iter().next(),
            env: intermediate_rep.env.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<WebrogueConfig> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<WebrogueConfig>> for HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<WebrogueConfig>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(format!(r#"Invalid header value for WebrogueConfig - value: {hdr_value} is invalid {e}"#))
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue> for header::IntoHeaderValue<WebrogueConfig> {
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <WebrogueConfig as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(format!(r#"Unable to convert header value '{value}' into WebrogueConfig - {err}"#))
                    }
             },
             std::result::Result::Err(e) => std::result::Result::Err(format!(r#"Unable to convert header: {hdr_value:?} to string: {e}"#))
        }
    }
}



/// Filesystem configuration
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct WebrogueConfigFilesystem {
    /// Readonly files and directories packaged with application
    #[serde(rename = "resources")]
          #[validate(nested)]
    #[serde(skip_serializing_if="Option::is_none")]
    pub resources: Option<Vec<models::WebrogueConfigFilesystemResourcesInner>>,

    /// Read-write volume's configuration.
    #[serde(rename = "persistent")]
          #[validate(nested)]
    #[serde(skip_serializing_if="Option::is_none")]
    pub persistent: Option<Vec<models::WebrogueConfigFilesystemPersistentInner>>,

}



impl WebrogueConfigFilesystem {
    #[allow(clippy::new_without_default, clippy::too_many_arguments)]
    pub fn new() -> WebrogueConfigFilesystem {
        WebrogueConfigFilesystem {
 resources: None,
 persistent: None,
        }
    }
}

/// Converts the WebrogueConfigFilesystem value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for WebrogueConfigFilesystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params: Vec<Option<String>> = vec![
            // Skipping resources in query parameter serialization

            // Skipping persistent in query parameter serialization

        ];

        write!(f, "{}", params.into_iter().flatten().collect::<Vec<_>>().join(","))
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a WebrogueConfigFilesystem value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for WebrogueConfigFilesystem {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub resources: Vec<Vec<models::WebrogueConfigFilesystemResourcesInner>>,
            pub persistent: Vec<Vec<models::WebrogueConfigFilesystemPersistentInner>>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing WebrogueConfigFilesystem".to_string())
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    "resources" => return std::result::Result::Err("Parsing a container in this style is not supported in WebrogueConfigFilesystem".to_string()),
                    "persistent" => return std::result::Result::Err("Parsing a container in this style is not supported in WebrogueConfigFilesystem".to_string()),
                    _ => return std::result::Result::Err("Unexpected key while parsing WebrogueConfigFilesystem".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(WebrogueConfigFilesystem {
            resources: intermediate_rep.resources.into_iter().next(),
            persistent: intermediate_rep.persistent.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<WebrogueConfigFilesystem> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<WebrogueConfigFilesystem>> for HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<WebrogueConfigFilesystem>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(format!(r#"Invalid header value for WebrogueConfigFilesystem - value: {hdr_value} is invalid {e}"#))
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue> for header::IntoHeaderValue<WebrogueConfigFilesystem> {
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <WebrogueConfigFilesystem as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(format!(r#"Unable to convert header value '{value}' into WebrogueConfigFilesystem - {err}"#))
                    }
             },
             std::result::Result::Err(e) => std::result::Result::Err(format!(r#"Unable to convert header: {hdr_value:?} to string: {e}"#))
        }
    }
}



/// Read-write volume configuration.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct WebrogueConfigFilesystemPersistentInner {
    /// Volume's name.
    #[serde(rename = "name")]
          #[validate(custom(function = "check_xss_string"))]
    pub name: String,

    /// Absolute path to volume. Application can refer volumes by this path.
    #[serde(rename = "mapped_path")]
          #[validate(custom(function = "check_xss_string"))]
    pub mapped_path: String,

}



impl WebrogueConfigFilesystemPersistentInner {
    #[allow(clippy::new_without_default, clippy::too_many_arguments)]
    pub fn new(name: String, mapped_path: String, ) -> WebrogueConfigFilesystemPersistentInner {
        WebrogueConfigFilesystemPersistentInner {
 name,
 mapped_path,
        }
    }
}

/// Converts the WebrogueConfigFilesystemPersistentInner value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for WebrogueConfigFilesystemPersistentInner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params: Vec<Option<String>> = vec![

            Some("name".to_string()),
            Some(self.name.to_string()),


            Some("mapped_path".to_string()),
            Some(self.mapped_path.to_string()),

        ];

        write!(f, "{}", params.into_iter().flatten().collect::<Vec<_>>().join(","))
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a WebrogueConfigFilesystemPersistentInner value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for WebrogueConfigFilesystemPersistentInner {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub name: Vec<String>,
            pub mapped_path: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing WebrogueConfigFilesystemPersistentInner".to_string())
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "name" => intermediate_rep.name.push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "mapped_path" => intermediate_rep.mapped_path.push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    _ => return std::result::Result::Err("Unexpected key while parsing WebrogueConfigFilesystemPersistentInner".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(WebrogueConfigFilesystemPersistentInner {
            name: intermediate_rep.name.into_iter().next().ok_or_else(|| "name missing in WebrogueConfigFilesystemPersistentInner".to_string())?,
            mapped_path: intermediate_rep.mapped_path.into_iter().next().ok_or_else(|| "mapped_path missing in WebrogueConfigFilesystemPersistentInner".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<WebrogueConfigFilesystemPersistentInner> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<WebrogueConfigFilesystemPersistentInner>> for HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<WebrogueConfigFilesystemPersistentInner>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(format!(r#"Invalid header value for WebrogueConfigFilesystemPersistentInner - value: {hdr_value} is invalid {e}"#))
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue> for header::IntoHeaderValue<WebrogueConfigFilesystemPersistentInner> {
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <WebrogueConfigFilesystemPersistentInner as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(format!(r#"Unable to convert header value '{value}' into WebrogueConfigFilesystemPersistentInner - {err}"#))
                    }
             },
             std::result::Result::Err(e) => std::result::Result::Err(format!(r#"Unable to convert header: {hdr_value:?} to string: {e}"#))
        }
    }
}



/// Readonly files and directories packaged with application
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct WebrogueConfigFilesystemResourcesInner {
    /// Relative path to file or directory you want to package.
    #[serde(rename = "real_path")]
          #[validate(custom(function = "check_xss_string"))]
    pub real_path: String,

    /// Absolute path to packaged file or directory. Application can refer is's resources by this path.
    #[serde(rename = "mapped_path")]
          #[validate(custom(function = "check_xss_string"))]
    pub mapped_path: String,

}



impl WebrogueConfigFilesystemResourcesInner {
    #[allow(clippy::new_without_default, clippy::too_many_arguments)]
    pub fn new(real_path: String, mapped_path: String, ) -> WebrogueConfigFilesystemResourcesInner {
        WebrogueConfigFilesystemResourcesInner {
 real_path,
 mapped_path,
        }
    }
}

/// Converts the WebrogueConfigFilesystemResourcesInner value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for WebrogueConfigFilesystemResourcesInner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params: Vec<Option<String>> = vec![

            Some("real_path".to_string()),
            Some(self.real_path.to_string()),


            Some("mapped_path".to_string()),
            Some(self.mapped_path.to_string()),

        ];

        write!(f, "{}", params.into_iter().flatten().collect::<Vec<_>>().join(","))
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a WebrogueConfigFilesystemResourcesInner value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for WebrogueConfigFilesystemResourcesInner {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub real_path: Vec<String>,
            pub mapped_path: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing WebrogueConfigFilesystemResourcesInner".to_string())
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "real_path" => intermediate_rep.real_path.push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "mapped_path" => intermediate_rep.mapped_path.push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    _ => return std::result::Result::Err("Unexpected key while parsing WebrogueConfigFilesystemResourcesInner".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(WebrogueConfigFilesystemResourcesInner {
            real_path: intermediate_rep.real_path.into_iter().next().ok_or_else(|| "real_path missing in WebrogueConfigFilesystemResourcesInner".to_string())?,
            mapped_path: intermediate_rep.mapped_path.into_iter().next().ok_or_else(|| "mapped_path missing in WebrogueConfigFilesystemResourcesInner".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<WebrogueConfigFilesystemResourcesInner> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<WebrogueConfigFilesystemResourcesInner>> for HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<WebrogueConfigFilesystemResourcesInner>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(format!(r#"Invalid header value for WebrogueConfigFilesystemResourcesInner - value: {hdr_value} is invalid {e}"#))
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue> for header::IntoHeaderValue<WebrogueConfigFilesystemResourcesInner> {
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <WebrogueConfigFilesystemResourcesInner as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(format!(r#"Unable to convert header value '{value}' into WebrogueConfigFilesystemResourcesInner - {err}"#))
                    }
             },
             std::result::Result::Err(e) => std::result::Result::Err(format!(r#"Unable to convert header: {hdr_value:?} to string: {e}"#))
        }
    }
}



/// Icon's configuration. This field is required to build for Android, macOS and iOS.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct WebrogueConfigIcons {
    #[serde(rename = "normal")]
          #[validate(nested)]
    pub normal: models::Icon,

}



impl WebrogueConfigIcons {
    #[allow(clippy::new_without_default, clippy::too_many_arguments)]
    pub fn new(normal: models::Icon, ) -> WebrogueConfigIcons {
        WebrogueConfigIcons {
 normal,
        }
    }
}

/// Converts the WebrogueConfigIcons value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for WebrogueConfigIcons {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params: Vec<Option<String>> = vec![
            // Skipping normal in query parameter serialization

        ];

        write!(f, "{}", params.into_iter().flatten().collect::<Vec<_>>().join(","))
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a WebrogueConfigIcons value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for WebrogueConfigIcons {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub normal: Vec<models::Icon>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing WebrogueConfigIcons".to_string())
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "normal" => intermediate_rep.normal.push(<models::Icon as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    _ => return std::result::Result::Err("Unexpected key while parsing WebrogueConfigIcons".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(WebrogueConfigIcons {
            normal: intermediate_rep.normal.into_iter().next().ok_or_else(|| "normal missing in WebrogueConfigIcons".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<WebrogueConfigIcons> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<WebrogueConfigIcons>> for HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<WebrogueConfigIcons>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(format!(r#"Invalid header value for WebrogueConfigIcons - value: {hdr_value} is invalid {e}"#))
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue> for header::IntoHeaderValue<WebrogueConfigIcons> {
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <WebrogueConfigIcons as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(format!(r#"Unable to convert header value '{value}' into WebrogueConfigIcons - {err}"#))
                    }
             },
             std::result::Result::Err(e) => std::result::Result::Err(format!(r#"Unable to convert header: {hdr_value:?} to string: {e}"#))
        }
    }
}


