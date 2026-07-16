use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs,
    io::Write,
    path::{Path, PathBuf},
};
use uuid::Uuid;

use crate::AppError;

fn schema_version() -> u16 {
    1
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Theme {
    #[serde(default = "schema_version")]
    pub schema_version: u16,
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub subtitle: String,
    #[serde(default)]
    pub tagline: String,
    #[serde(default)]
    pub project_prefix: String,
    #[serde(default)]
    pub project_label: String,
    #[serde(default)]
    pub status_text: String,
    #[serde(default)]
    pub quote: String,
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
    #[serde(default)]
    pub image_position_x_pct: Option<u16>,
    #[serde(default)]
    pub image_position_y_pct: Option<u16>,
    #[serde(default)]
    pub image_scale_pct: Option<u16>,
    #[serde(default)]
    pub panel_opacity: Option<f32>,
    #[serde(default)]
    pub panel_blur_px: Option<u16>,
    #[serde(default)]
    pub corner_radius_px: Option<u16>,
    #[serde(default)]
    pub content_max_width_px: Option<u16>,
    #[serde(default)]
    pub show_brand: Option<bool>,
    #[serde(default)]
    pub show_status: Option<bool>,
    #[serde(default)]
    pub show_quote: Option<bool>,
    #[serde(default)]
    pub show_orbit: Option<bool>,
    #[serde(default)]
    pub show_particles: Option<bool>,
    #[serde(default)]
    pub particle_count: Option<u8>,
}

impl Theme {
    pub fn validate(&self) -> Result<(), AppError> {
        if self.schema_version != 1 {
            return Err(AppError::Validation("仅支持版本 1 的主题文件".into()));
        }
        if self.id.is_empty()
            || self.id.len() > 80
            || !self
                .id
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_'))
        {
            return Err(AppError::Validation(
                "主题 ID 只能包含 1–80 个英文字母、数字、短横线或下划线".into(),
            ));
        }
        validate_text("主题名称", &self.name, 120, false)?;
        validate_text("副标题", &self.subtitle, 80, true)?;
        validate_text("主题说明", &self.tagline, 160, true)?;
        validate_text("项目名前缀", &self.project_prefix, 80, true)?;
        validate_text("项目栏标题", &self.project_label, 80, true)?;
        validate_text("状态文字", &self.status_text, 80, true)?;
        validate_text("引言", &self.quote, 80, true)?;

        const ALLOWED_COLORS: [&str; 10] = [
            "background",
            "panel",
            "panel-alt",
            "accent",
            "accent-alt",
            "secondary",
            "highlight",
            "text",
            "muted",
            "line",
        ];
        if self.colors.len() > ALLOWED_COLORS.len() {
            return Err(AppError::Validation("主题颜色数量过多".into()));
        }
        for (key, value) in &self.colors {
            if !ALLOWED_COLORS.contains(&key.as_str()) {
                return Err(AppError::Validation(format!("不支持的颜色字段：{key}")));
            }
            if !valid_hex_color(value) {
                return Err(AppError::Validation(format!("颜色 {key} 不是有效的十六进制颜色")));
            }
        }
        bounded_float("背景透明度", self.opacity, 0.0, 1.0)?;
        bounded_u16("背景模糊", self.blur_px, 0, 100)?;
        bounded_u16("背景亮度", self.brightness_pct, 10, 200)?;
        bounded_u16("背景饱和度", self.saturation_pct, 0, 300)?;
        bounded_u16("图片横向位置", self.image_position_x_pct, 0, 100)?;
        bounded_u16("图片纵向位置", self.image_position_y_pct, 0, 100)?;
        bounded_u16("图片缩放", self.image_scale_pct, 50, 200)?;
        bounded_float("面板透明度", self.panel_opacity, 0.0, 1.0)?;
        bounded_u16("面板模糊", self.panel_blur_px, 0, 60)?;
        bounded_u16("圆角", self.corner_radius_px, 0, 40)?;
        bounded_u16("内容最大宽度", self.content_max_width_px, 640, 1600)?;
        if self.particle_count.is_some_and(|v| v > 24) {
            return Err(AppError::Validation("粒子数量不能超过 24".into()));
        }
        if let Some(image) = &self.background_image {
            let p = Path::new(image);
            if p.is_absolute()
                || image.contains("..")
                || image.contains('\0')
                || image.contains('\\')
                || !image.starts_with("images/")
            {
                return Err(AppError::Validation(
                    "背景图片必须是已导入的 images/<文件名> 路径".into(),
                ));
            }
        }
        Ok(())
    }

    pub fn shareable(&self) -> Self {
        let mut copy = self.clone();
        copy.background_image = None;
        copy
    }
}

fn validate_text(label: &str, value: &str, max: usize, allow_empty: bool) -> Result<(), AppError> {
    if (!allow_empty && value.trim().is_empty()) || value.chars().count() > max || value.contains('\0') {
        return Err(AppError::Validation(format!("{label}长度不符合要求")));
    }
    Ok(())
}

fn bounded_float(label: &str, value: Option<f32>, min: f32, max: f32) -> Result<(), AppError> {
    if value.is_some_and(|v| !v.is_finite() || !(min..=max).contains(&v)) {
        return Err(AppError::Validation(format!("{label}必须在 {min} 到 {max} 之间")));
    }
    Ok(())
}

fn bounded_u16(label: &str, value: Option<u16>, min: u16, max: u16) -> Result<(), AppError> {
    if value.is_some_and(|v| !(min..=max).contains(&v)) {
        return Err(AppError::Validation(format!("{label}必须在 {min} 到 {max} 之间")));
    }
    Ok(())
}

fn valid_hex_color(value: &str) -> bool {
    matches!(value.len(), 4 | 5 | 7 | 9)
        && value.starts_with('#')
        && value[1..].chars().all(|c| c.is_ascii_hexdigit())
}

#[derive(Clone)]
pub struct ThemeStore {
    root: PathBuf,
}

impl ThemeStore {
    pub fn new(root: PathBuf) -> Result<Self, AppError> {
        fs::create_dir_all(root.join("themes"))?;
        fs::create_dir_all(root.join("images"))?;
        Ok(Self { root })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn list(&self) -> Result<Vec<Theme>, AppError> {
        let mut result = Vec::new();
        for entry in fs::read_dir(self.root.join("themes"))? {
            let path = entry?.path();
            if path.extension().and_then(|v| v.to_str()) == Some("json") {
                if let Ok(theme) = read_theme_file(&path) {
                    result.push(theme);
                }
            }
        }
        result.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(result)
    }

    pub fn save(&self, theme: &Theme) -> Result<(), AppError> {
        theme.validate()?;
        let target = self
            .root
            .join("themes")
            .join(format!("{}.json", theme.id));
        write_atomic_json(&target, theme)
    }

    pub fn delete(&self, id: &str) -> Result<(), AppError> {
        validate_theme_id(id)?;
        let path = self.root.join("themes").join(format!("{id}.json"));
        if path.exists() {
            fs::remove_file(path)?;
        }
        Ok(())
    }
}

pub fn read_theme_file(path: &Path) -> Result<Theme, AppError> {
    let meta = fs::metadata(path)?;
    if !meta.is_file() || meta.len() > 256 * 1024 {
        return Err(AppError::Validation("主题文件必须小于 256 KiB".into()));
    }
    let theme: Theme = serde_json::from_slice(&fs::read(path)?)?;
    theme.validate()?;
    Ok(theme)
}

pub fn export_theme_file(path: &Path, theme: &Theme) -> Result<(), AppError> {
    theme.validate()?;
    write_atomic_json(path, &theme.shareable())
}

fn write_atomic_json(path: &Path, theme: &Theme) -> Result<(), AppError> {
    let parent = path
        .parent()
        .ok_or_else(|| AppError::Validation("导出路径无效".into()))?;
    fs::create_dir_all(parent)?;
    let temp = parent.join(format!(".csw-theme-{}.tmp", Uuid::new_v4()));
    let bytes = serde_json::to_vec_pretty(theme)?;
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temp)?;
    file.write_all(&bytes)?;
    file.sync_all()?;
    drop(file);
    if path.exists() {
        fs::remove_file(path)?;
    }
    fs::rename(temp, path)?;
    Ok(())
}

fn validate_theme_id(id: &str) -> Result<(), AppError> {
    if id.is_empty()
        || id.len() > 80
        || !id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_'))
    {
        return Err(AppError::Validation("主题 ID 无效".into()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_theme() -> Theme {
        Theme {
            schema_version: 1,
            id: "night".into(),
            name: "夜色".into(),
            subtitle: "CODEX SKIN".into(),
            tagline: "把喜欢的画面变成工作台".into(),
            project_prefix: "项目 · ".into(),
            project_label: "选择项目".into(),
            status_text: "主题已启用".into(),
            quote: "创造美好的东西".into(),
            colors: BTreeMap::from([
                ("accent".into(), "#abcdef".into()),
                ("background".into(), "#101319".into()),
            ]),
            background_image: Some("images/background.jpg".into()),
            opacity: Some(0.8),
            blur_px: Some(4),
            brightness_pct: Some(80),
            saturation_pct: Some(120),
            image_position_x_pct: Some(50),
            image_position_y_pct: Some(50),
            image_scale_pct: Some(100),
            panel_opacity: Some(0.86),
            panel_blur_px: Some(12),
            corner_radius_px: Some(18),
            content_max_width_px: Some(950),
            show_brand: Some(true),
            show_status: Some(true),
            show_quote: Some(true),
            show_orbit: Some(true),
            show_particles: Some(true),
            particle_count: Some(8),
        }
    }

    #[test]
    fn rejects_code_fields_and_bad_colors() {
        let raw = r##"{"schema_version":1,"id":"x","name":"x","colors":{},"css":"body{}"}"##;
        assert!(serde_json::from_str::<Theme>(raw).is_err());
        let mut theme = sample_theme();
        theme.colors.insert("accent".into(), "red; color:white".into());
        assert!(theme.validate().is_err());
    }

    #[test]
    fn old_versionless_theme_defaults_to_version_one() {
        let raw = r##"{"id":"x","name":"x","colors":{"accent":"#abcdef"}}"##;
        let theme: Theme = serde_json::from_str(raw).unwrap();
        assert_eq!(theme.schema_version, 1);
        assert!(theme.validate().is_ok());
    }

    #[test]
    fn exported_theme_does_not_leak_local_image_path() {
        let theme = sample_theme();
        assert_eq!(theme.shareable().background_image, None);
    }

    #[test]
    fn round_trip_store() {
        let dir = tempfile::tempdir().unwrap();
        let store = ThemeStore::new(dir.path().to_owned()).unwrap();
        let theme = sample_theme();
        store.save(&theme).unwrap();
        assert_eq!(store.list().unwrap(), vec![theme]);
        store.delete("night").unwrap();
        assert!(store.list().unwrap().is_empty());
    }
}
