use serde::{Deserialize, Serialize};
use thiserror::Error;

/// The exact persisted keys returned by LaunchBox 13.27's protected
/// `ModelType` accessors.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub enum ModelType {
    Box,
    DvdCase,
    JewelCase,
    LongJewelCase,
    /// Retained so a newer LaunchBox value can be round-tripped without being
    /// silently rewritten by this version of the port.
    Unknown(String),
}

impl ModelType {
    pub const fn key(&self) -> &str {
        match self {
            Self::Box => "box",
            Self::DvdCase => "dvd",
            Self::JewelCase => "jewelCase",
            Self::LongJewelCase => "longJewelCase",
            Self::Unknown(value) => value.as_str(),
        }
    }

    pub const fn display_name(&self) -> &str {
        match self {
            Self::Box => "Box",
            Self::DvdCase => "DVD Case",
            Self::JewelCase => "Jewel Case",
            Self::LongJewelCase => "Long Jewel Case",
            Self::Unknown(_) => "Unknown Model",
        }
    }

    pub fn from_key(value: &str) -> Self {
        match value {
            "box" => Self::Box,
            "dvd" => Self::DvdCase,
            "jewelCase" => Self::JewelCase,
            "longJewelCase" => Self::LongJewelCase,
            value => Self::Unknown(value.to_string()),
        }
    }
}

impl From<String> for ModelType {
    fn from(value: String) -> Self {
        Self::from_key(&value)
    }
}

impl From<ModelType> for String {
    fn from(value: ModelType) -> Self {
        value.key().to_string()
    }
}

/// LaunchBox stores `System.Drawing.Color.ToArgb()` as a signed 32-bit XML
/// integer. Keeping that representation avoids channel-order ambiguity.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ArgbColor(i32);

impl ArgbColor {
    pub const fn from_raw(value: i32) -> Self {
        Self(value)
    }

    pub const fn from_channels(alpha: u8, red: u8, green: u8, blue: u8) -> Self {
        Self(((alpha as u32) << 24 | (red as u32) << 16 | (green as u32) << 8 | blue as u32) as i32)
    }

    pub const fn raw(self) -> i32 {
        self.0
    }

    /// Parses the color notation used at the CXX-Qt presentation boundary.
    ///
    /// LaunchBox itself still persists the signed [`Self::raw`] value. The
    /// six-digit form is accepted for editor convenience and receives a fully
    /// opaque alpha channel.
    pub fn parse_qt_hex(value: &str) -> Result<Self, ModelSettingsError> {
        let hexadecimal = value.strip_prefix('#').unwrap_or(value);
        let raw = match hexadecimal.len() {
            6 => u32::from_str_radix(hexadecimal, 16)
                .map(|rgb| 0xff00_0000 | rgb)
                .map_err(|_| ModelSettingsError::InvalidArgbColor {
                    value: value.to_string(),
                })?,
            8 => u32::from_str_radix(hexadecimal, 16).map_err(|_| {
                ModelSettingsError::InvalidArgbColor {
                    value: value.to_string(),
                }
            })?,
            _ => {
                return Err(ModelSettingsError::InvalidArgbColor {
                    value: value.to_string(),
                });
            }
        };
        Ok(Self(raw as i32))
    }

    pub const fn alpha(self) -> u8 {
        ((self.0 as u32) >> 24) as u8
    }

    pub const fn red(self) -> u8 {
        ((self.0 as u32) >> 16) as u8
    }

    pub const fn green(self) -> u8 {
        ((self.0 as u32) >> 8) as u8
    }

    pub const fn blue(self) -> u8 {
        self.0 as u8
    }

    /// Qt accepts `#AARRGGBB`, so this preserves alpha as well as RGB.
    pub fn qt_hex(self) -> String {
        format!("#{:08x}", self.0 as u32)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct ModelSize {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl ModelSize {
    pub fn parse_launchbox(value: &str) -> Result<Self, ModelSettingsError> {
        let values = value.split(';').collect::<Vec<_>>();
        if values.len() != 3 {
            return Err(ModelSettingsError::InvalidModelSize {
                value: value.to_string(),
            });
        }
        let parse = |part: &str| {
            part.parse::<f64>()
                .map_err(|_| ModelSettingsError::InvalidModelSize {
                    value: value.to_string(),
                })
        };
        let size = Self {
            x: parse(values[0])?,
            y: parse(values[1])?,
            z: parse(values[2])?,
        };
        size.validate()?;
        Ok(size)
    }

    pub fn to_launchbox(self) -> String {
        format!("{};{};{}", self.x, self.y, self.z)
    }

    pub fn validate(self) -> Result<(), ModelSettingsError> {
        if [self.x, self.y, self.z]
            .into_iter()
            .all(|value| value.is_finite() && value > 0.0)
        {
            Ok(())
        } else {
            Err(ModelSettingsError::InvalidModelSize {
                value: self.to_launchbox(),
            })
        }
    }
}

/// One root-level `<ModelSettings>` record in a LaunchBox XML document.
///
/// Rotation strings remain lossless because the protected implementation uses
/// sparse comma-separated side slots whose interpretation differs by model
/// type. They are not host paths and must not be normalized.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ModelSettings {
    pub case_color: Option<ArgbColor>,
    pub cover_color: Option<ArgbColor>,
    pub front_spine_image: Option<String>,
    pub front_spine_is_clear: bool,
    pub full_image_spine_width: f64,
    pub full_scan_is_landscape: bool,
    pub game_id: Option<String>,
    pub logo_font: Option<String>,
    pub logo_rotation: String,
    pub model_size: Option<ModelSize>,
    pub model_type: Option<ModelType>,
    pub platform_name: Option<String>,
    pub spine_rotation: String,
    pub use_full_scan_images: bool,
}

impl Default for ModelSettings {
    fn default() -> Self {
        Self {
            case_color: None,
            cover_color: None,
            front_spine_image: None,
            front_spine_is_clear: false,
            full_image_spine_width: 0.143,
            full_scan_is_landscape: false,
            game_id: None,
            logo_font: None,
            logo_rotation: "0,0,0,".to_string(),
            model_size: None,
            model_type: None,
            platform_name: None,
            spine_rotation: "0,,0,".to_string(),
            use_full_scan_images: false,
        }
    }
}

impl ModelSettings {
    pub fn box_defaults() -> Self {
        Self {
            full_image_spine_width: 0.088,
            model_type: Some(ModelType::Box),
            use_full_scan_images: true,
            ..Self::default()
        }
    }

    pub fn dvd_defaults() -> Self {
        Self {
            full_image_spine_width: 0.065,
            logo_rotation: "0,,,".to_string(),
            model_type: Some(ModelType::DvdCase),
            spine_rotation: "0,,,".to_string(),
            use_full_scan_images: true,
            ..Self::default()
        }
    }

    pub fn jewel_case_defaults() -> Self {
        Self {
            logo_rotation: "0,,0,".to_string(),
            model_type: Some(ModelType::JewelCase),
            ..Self::default()
        }
    }

    pub fn long_jewel_case_defaults() -> Self {
        Self {
            logo_rotation: "0,,0,".to_string(),
            model_type: Some(ModelType::LongJewelCase),
            ..Self::default()
        }
    }

    pub fn validate(&self) -> Result<(), ModelSettingsError> {
        if !self.full_image_spine_width.is_finite()
            || !(0.0..=1.0).contains(&self.full_image_spine_width)
        {
            return Err(ModelSettingsError::InvalidSpineWidth {
                value: self.full_image_spine_width,
            });
        }
        if let Some(size) = self.model_size {
            size.validate()?;
        }
        if self.game_id.as_deref().is_some_and(str::is_empty) {
            return Err(ModelSettingsError::EmptyIdentity { field: "GameId" });
        }
        if self.platform_name.as_deref().is_some_and(str::is_empty) {
            return Err(ModelSettingsError::EmptyIdentity {
                field: "PlatformName",
            });
        }
        if self.game_id.is_some() && self.platform_name.is_some() {
            return Err(ModelSettingsError::ConflictingIdentity);
        }
        Ok(())
    }

    pub fn effective_model_type(&self) -> &ModelType {
        self.model_type.as_ref().unwrap_or(&ModelType::Box)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ModelSettingsSource {
    GameOverride,
    PlatformOverride,
    BuiltInPlatform,
    BoxFallback,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ResolvedModelSettings {
    pub source: ModelSettingsSource,
    pub settings: ModelSettings,
}

/// LaunchBox resolves a complete game override before a complete platform
/// override. Its protected built-in lookup checks the platform name first,
/// then `ScrapeAs`; an unmapped pair returns null and is rendered as a box.
pub fn resolve_model_settings(
    game_id: &str,
    platform_name: &str,
    scrape_as: Option<&str>,
    game_records: &[ModelSettings],
    platform_records: &[ModelSettings],
) -> ResolvedModelSettings {
    if let Some(settings) = game_records
        .iter()
        .find(|settings| settings.game_id.as_deref() == Some(game_id))
    {
        return ResolvedModelSettings {
            source: ModelSettingsSource::GameOverride,
            settings: settings.clone(),
        };
    }
    if let Some(settings) = platform_records
        .iter()
        .find(|settings| settings.platform_name.as_deref() == Some(platform_name))
    {
        return ResolvedModelSettings {
            source: ModelSettingsSource::PlatformOverride,
            settings: settings.clone(),
        };
    }
    if let Some(settings) = built_in_model_settings(platform_name, scrape_as) {
        return ResolvedModelSettings {
            source: ModelSettingsSource::BuiltInPlatform,
            settings,
        };
    }
    ResolvedModelSettings {
        source: ModelSettingsSource::BoxFallback,
        settings: ModelSettings::box_defaults(),
    }
}

pub const BUILT_IN_MODEL_PLATFORM_NAMES: [&str; 41] = [
    "3DO Interactive Multiplayer",
    "Amstrad GX4000",
    "Commodore Amiga CD32",
    "Commodore CDTV",
    "Exelvision EXL 100",
    "Game Wave Family Entertainment System",
    "Microsoft Xbox",
    "Microsoft Xbox 360",
    "Microsoft Xbox One",
    "NEC PC-8801",
    "NEC PC-9801",
    "NEC PC-FX",
    "NEC TurboGrafx-CD",
    "Nintendo 3DS",
    "Nintendo DS",
    "Nintendo GameCube",
    "Nintendo Switch",
    "Nintendo Wii",
    "Nintendo Wii U",
    "Nokia N-Gage",
    "Nuon",
    "Philips CD-i",
    "Sega CD",
    "Sega Dreamcast",
    "Sega Genesis",
    "Sega Master System",
    "Sega Saturn",
    "Sharp X1",
    "Sharp X68000",
    "SNK Neo Geo AES",
    "SNK Neo Geo CD",
    "SNK Neo Geo Pocket",
    "SNK Neo Geo Pocket Color",
    "Sony Playstation",
    "Sony Playstation 2",
    "Sony Playstation 3",
    "Sony Playstation 4",
    "Sony Playstation Vita",
    "Sony PSP",
    "Tapwave Zodiac",
    "Windows",
];

pub fn built_in_model_settings(
    platform_name: &str,
    scrape_as: Option<&str>,
) -> Option<ModelSettings> {
    built_in_model_settings_exact(platform_name)
        .or_else(|| scrape_as.and_then(built_in_model_settings_exact))
}

fn built_in_model_settings_exact(name: &str) -> Option<ModelSettings> {
    let mut settings = match name {
        "Commodore Amiga CD32"
        | "Commodore CDTV"
        | "NEC TurboGrafx-CD"
        | "Philips CD-i"
        | "Sega Dreamcast"
        | "SNK Neo Geo CD"
        | "Sony Playstation" => ModelSettings::jewel_case_defaults(),
        "Sega CD" | "Sega Saturn" => ModelSettings::long_jewel_case_defaults(),
        name if BUILT_IN_MODEL_PLATFORM_NAMES.contains(&name) => ModelSettings {
            // The built-in DVD records retain the base constructor's sparse
            // rotation strings rather than the editor's freshly emitted ones.
            full_image_spine_width: 0.065,
            model_type: Some(ModelType::DvdCase),
            use_full_scan_images: true,
            ..ModelSettings::default()
        },
        _ => return None,
    };

    settings.case_color = match name {
        "Microsoft Xbox" => Some(ArgbColor::from_channels(255, 12, 133, 12)),
        "Microsoft Xbox 360" => Some(ArgbColor::from_channels(255, 94, 173, 87)),
        "Microsoft Xbox One" => Some(ArgbColor::from_channels(255, 16, 124, 16)),
        "Nintendo 3DS" | "Nintendo Switch" | "Nintendo Wii" | "Sony Playstation 3" | "Sony PSP" => {
            Some(ArgbColor::from_channels(255, 245, 245, 245))
        }
        "Nintendo Wii U" => Some(ArgbColor::from_channels(255, 0, 154, 199)),
        "Sony Playstation 4" | "Sony Playstation Vita" => {
            Some(ArgbColor::from_channels(255, 30, 81, 206))
        }
        _ => None,
    };
    settings.cover_color = match name {
        "Sega Master System" => Some(ArgbColor::from_channels(255, 230, 230, 230)),
        _ => None,
    };
    settings.model_size = match name {
        "Sega Genesis" | "Sega Master System" => Some(ModelSize {
            x: 5.0,
            y: 7.165,
            z: 1.0,
        }),
        _ => None,
    };
    match name {
        "Sega Dreamcast" | "Sony Playstation" => {
            settings.front_spine_image = Some(format!("{{Resources}}\\{name}"));
            settings.front_spine_is_clear = true;
        }
        _ => {}
    }
    Some(settings)
}

#[derive(Clone, Debug, Error, PartialEq)]
pub enum ModelSettingsError {
    #[error("ARGB color must use #AARRGGBB or #RRGGBB notation: {value}")]
    InvalidArgbColor { value: String },
    #[error("model size must contain three positive finite semicolon-separated numbers: {value}")]
    InvalidModelSize { value: String },
    #[error("full-scan spine width must be finite and between 0 and 1, got {value}")]
    InvalidSpineWidth { value: f64 },
    #[error("{field} must not be empty when present")]
    EmptyIdentity { field: &'static str },
    #[error("a model-settings record cannot target both a game and a platform")]
    ConflictingIdentity,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_model_type_keys_round_trip_and_future_values_survive() {
        for (key, expected) in [
            ("box", ModelType::Box),
            ("dvd", ModelType::DvdCase),
            ("jewelCase", ModelType::JewelCase),
            ("longJewelCase", ModelType::LongJewelCase),
        ] {
            assert_eq!(ModelType::from_key(key), expected);
            assert_eq!(expected.key(), key);
        }
        assert_eq!(
            ModelType::from_key("futureCase"),
            ModelType::Unknown("futureCase".to_string())
        );
    }

    #[test]
    fn argb_is_lossless_and_qt_ready() {
        let color = ArgbColor::from_raw(-15_654_349);
        assert_eq!(
            (color.alpha(), color.red(), color.green(), color.blue()),
            (255, 17, 34, 51)
        );
        assert_eq!(color.qt_hex(), "#ff112233");
        assert_eq!(color.raw(), -15_654_349);
        assert_eq!(ArgbColor::parse_qt_hex("#ff112233").unwrap(), color);
        assert_eq!(ArgbColor::parse_qt_hex("112233").unwrap(), color);
        assert!(ArgbColor::parse_qt_hex("#xyz").is_err());
        assert!(ArgbColor::parse_qt_hex("#0011223344").is_err());
    }

    #[test]
    fn launchbox_model_size_uses_semicolons() {
        let size = ModelSize::parse_launchbox("5;7.165;1").expect("parse oracle value");
        assert_eq!(
            size,
            ModelSize {
                x: 5.0,
                y: 7.165,
                z: 1.0
            }
        );
        assert_eq!(size.to_launchbox(), "5;7.165;1");
        assert!(ModelSize::parse_launchbox("5,7.165,1").is_err());
        assert!(ModelSize::parse_launchbox("5;0;1").is_err());
    }

    #[test]
    fn all_41_oracle_platform_defaults_resolve() {
        assert_eq!(BUILT_IN_MODEL_PLATFORM_NAMES.len(), 41);
        for name in BUILT_IN_MODEL_PLATFORM_NAMES {
            let settings = built_in_model_settings(name, None)
                .unwrap_or_else(|| panic!("missing built-in model settings for {name}"));
            settings.validate().expect("valid oracle settings");
        }
        assert_eq!(
            built_in_model_settings("Unknown", Some("Sony Playstation"))
                .expect("ScrapeAs fallback")
                .effective_model_type(),
            &ModelType::JewelCase
        );
        assert!(built_in_model_settings("Unknown", Some("Unknown")).is_none());
    }

    #[test]
    fn platform_specific_oracle_values_are_exact() {
        let playstation =
            built_in_model_settings("Sony Playstation", None).expect("PlayStation settings");
        assert_eq!(playstation.model_type, Some(ModelType::JewelCase));
        assert_eq!(
            playstation.front_spine_image.as_deref(),
            Some(r"{Resources}\Sony Playstation")
        );
        assert!(playstation.front_spine_is_clear);
        assert!(!playstation.use_full_scan_images);

        let master_system =
            built_in_model_settings("Sega Master System", None).expect("SMS settings");
        assert_eq!(master_system.model_type, Some(ModelType::DvdCase));
        assert_eq!(
            master_system.model_size,
            Some(ModelSize {
                x: 5.0,
                y: 7.165,
                z: 1.0
            })
        );
        assert_eq!(
            master_system.cover_color,
            Some(ArgbColor::from_channels(255, 230, 230, 230))
        );
    }

    #[test]
    fn override_precedence_matches_launchbox_contract() {
        let platform = ModelSettings {
            platform_name: Some("Sony Playstation".to_string()),
            model_type: Some(ModelType::DvdCase),
            ..ModelSettings::default()
        };
        let game = ModelSettings {
            game_id: Some("game-1".to_string()),
            model_type: Some(ModelType::LongJewelCase),
            ..ModelSettings::default()
        };
        let resolved = resolve_model_settings(
            "game-1",
            "Sony Playstation",
            None,
            std::slice::from_ref(&game),
            std::slice::from_ref(&platform),
        );
        assert_eq!(resolved.source, ModelSettingsSource::GameOverride);
        assert_eq!(
            resolved.settings.effective_model_type(),
            &ModelType::LongJewelCase
        );

        let platform_resolved =
            resolve_model_settings("game-2", "Sony Playstation", None, &[], &[platform]);
        assert_eq!(
            platform_resolved.source,
            ModelSettingsSource::PlatformOverride
        );
        assert_eq!(
            platform_resolved.settings.effective_model_type(),
            &ModelType::DvdCase
        );

        let fallback = resolve_model_settings("game-3", "Unknown", None, &[], &[]);
        assert_eq!(fallback.source, ModelSettingsSource::BoxFallback);
        assert_eq!(fallback.settings.model_type, Some(ModelType::Box));
    }
}
