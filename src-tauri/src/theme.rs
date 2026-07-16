use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fs, io::Write, path::{Path, PathBuf}};
use uuid::Uuid;

use crate::AppError;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Theme {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub colors: BTreeMap<String, String>,
    #[serde(default)]
    pub background_image: Option<String>,
    #[serde(default)]
    pub opacity: Option<f32>,
    #[serde(default)]
    pub blur_px: Option<u16>,
    #[serde(default)]
    pub brightness_pct: Option<u16>,
    #[serde(default)]
    pub saturation_pct: Option<u16>,
}

impl Theme {
    pub fn validate(&self) -> Result<(), AppError> {
        if self.id.is_empty() || self.id.len() > 80 || !self.id.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_')) {
            return Err(AppError::Validation("theme id must be 1-80 ASCII letters, digits, '-' or '_'".into()));
        }
        if self.name.trim().is_empty() || self.name.len() > 120 {
            return Err(AppError::Validation("theme name must be 1-120 characters".into()));
        }
        if self.colors.len() > 64 {
            return Err(AppError::Validation("too many theme colors".into()));
        }
        for (key, value) in &self.colors {
            if key.is_empty() || key.len() > 48 || !key.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
                return Err(AppError::Validation(format!("invalid color key: {key}")));
            }
            if !valid_hex_color(value) {
                return Err(AppError::Validation(format!("invalid color value for {key}")));
            }
        }
        if self.opacity.is_some_and(|v| !v.is_finite() || !(0.0..=1.0).contains(&v)) {
            return Err(AppError::Validation("opacity must be between 0 and 1".into()));
        }
        if self.blur_px.is_some_and(|v| v > 100) {
            return Err(AppError::Validation("blur_px must not exceed 100".into()));
        }
        if self.brightness_pct.is_some_and(|v| !(10..=200).contains(&v)) {
            return Err(AppError::Validation("brightness_pct must be between 10 and 200".into()));
        }
        if self.saturation_pct.is_some_and(|v| v > 300) {
            return Err(AppError::Validation("saturation_pct must not exceed 300".into()));
        }
        if let Some(image) = &self.background_image {
            let p = Path::new(image);
            if p.is_absolute() || image.contains("..") || image.contains('\0') || image.contains('\\') || !image.starts_with("images/") {
                return Err(AppError::Validation("background_image must be an imported images/<name> path".into()));
            }
        }
        Ok(())
    }
}

fn valid_hex_color(value: &str) -> bool {
    matches!(value.len(), 4 | 5 | 7 | 9) && value.starts_with('#') && value[1..].chars().all(|c| c.is_ascii_hexdigit())
}

#[derive(Clone)]
pub struct ThemeStore { root: PathBuf }

impl ThemeStore {
    pub fn new(root: PathBuf) -> Result<Self, AppError> {
        fs::create_dir_all(root.join("themes"))?;
        fs::create_dir_all(root.join("images"))?;
        Ok(Self { root })
    }

    pub fn root(&self) -> &Path { &self.root }

    pub fn list(&self) -> Result<Vec<Theme>, AppError> {
        let mut result = Vec::new();
        for entry in fs::read_dir(self.root.join("themes"))? {
            let path = entry?.path();
            if path.extension().and_then(|v| v.to_str()) == Some("json") {
                let theme: Theme = serde_json::from_slice(&fs::read(path)?)?;
                theme.validate()?;
                result.push(theme);
            }
        }
        result.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(result)
    }

    pub fn save(&self, theme: &Theme) -> Result<(), AppError> {
        theme.validate()?;
        let target = self.root.join("themes").join(format!("{}.json", theme.id));
        let temp = self.root.join("themes").join(format!(".{}.{}.tmp", theme.id, Uuid::new_v4()));
        let bytes = serde_json::to_vec_pretty(theme)?;
        let mut file = fs::OpenOptions::new().write(true).create_new(true).open(&temp)?;
        file.write_all(&bytes)?;
        file.sync_all()?;
        drop(file);
        if target.exists() { fs::remove_file(&target)?; }
        fs::rename(temp, target)?;
        Ok(())
    }

    pub fn delete(&self, id: &str) -> Result<(), AppError> {
        let probe = Theme { id: id.into(), name: "probe".into(), colors: BTreeMap::new(), background_image: None, opacity: None, blur_px: None, brightness_pct: None, saturation_pct: None };
        probe.validate()?;
        let path = self.root.join("themes").join(format!("{id}.json"));
        if path.exists() { fs::remove_file(path)?; }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_code_fields_and_bad_colors() {
        let raw = r##"{"id":"x","name":"x","colors":{},"css":"body{}"}"##;
        assert!(serde_json::from_str::<Theme>(raw).is_err());
        let mut t = Theme { id: "x".into(), name: "X".into(), colors: BTreeMap::new(), background_image: None, opacity: None, blur_px: None, brightness_pct: None, saturation_pct: None };
        t.colors.insert("accent".into(), "red; color:white".into());
        assert!(t.validate().is_err());
    }

    #[test]
    fn round_trip_store() {
        let dir = tempfile::tempdir().unwrap();
        let store = ThemeStore::new(dir.path().to_owned()).unwrap();
        let t = Theme { id: "night".into(), name: "Night".into(), colors: BTreeMap::from([("accent".into(), "#abcdef".into())]), background_image: None, opacity: Some(0.8), blur_px: Some(4), brightness_pct: Some(80), saturation_pct: Some(120) };
        store.save(&t).unwrap();
        assert_eq!(store.list().unwrap(), vec![t]);
        store.delete("night").unwrap();
        assert!(store.list().unwrap().is_empty());
    }
}
