use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct ThemeConfig {
    pub radius: f32,
    pub shadow: bool,
    pub font: String,
    pub font_size: f32,
    pub background: String,
    pub text_color: String,
    pub highlight_color: String,
    pub highlight_text_color: String,
    pub comment_color: String,
    pub animation: String,
    pub border_width: f32,
    pub border_color: String,
    pub candidate_spacing: f32,
    pub padding: f32,
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            radius: 12.0, shadow: true, font: "Segoe UI".into(), font_size: 14.0,
            background: "#202124".into(), text_color: "#E8EAED".into(),
            highlight_color: "#8AB4F8".into(), highlight_text_color: "#202124".into(),
            comment_color: "#9AA0A6".into(), animation: "spring".into(),
            border_width: 0.0, border_color: "#5F6368".into(),
            candidate_spacing: 4.0, padding: 8.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::theme::ThemeConfig;

    #[test]
    fn test_theme_defaults() {
        let t = ThemeConfig::default();
        assert_eq!(t.radius, 12.0);
        assert!(t.shadow);
        assert_eq!(t.font, "Segoe UI");
        assert_eq!(t.font_size, 14.0);
        assert_eq!(t.background, "#202124");
        assert_eq!(t.text_color, "#E8EAED");
        assert_eq!(t.highlight_color, "#8AB4F8");
        assert_eq!(t.highlight_text_color, "#202124");
        assert_eq!(t.comment_color, "#9AA0A6");
        assert_eq!(t.animation, "spring");
        assert_eq!(t.border_width, 0.0);
        assert_eq!(t.border_color, "#5F6368");
        assert_eq!(t.candidate_spacing, 4.0);
        assert_eq!(t.padding, 8.0);
    }

    #[test]
    fn test_theme_serde_roundtrip() {
        let t = ThemeConfig::default();
        let json = serde_json::to_string(&t).unwrap();
        let t2: ThemeConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(t2.radius, t.radius);
        assert_eq!(t2.font, t.font);
        assert_eq!(t2.background, t.background);
    }

    #[test]
    fn test_theme_partial_json() {
        let json = r#"{"radius": 16, "font": "Noto Sans"}"#;
        let t: ThemeConfig = serde_json::from_str(json).unwrap();
        assert_eq!(t.radius, 16.0);
        assert_eq!(t.font, "Noto Sans");
        assert!(t.shadow); // default
    }

    #[test]
    fn test_theme_empty_json() {
        let json = "{}";
        let t: ThemeConfig = serde_json::from_str(json).unwrap();
        assert_eq!(t.radius, 12.0); // default
    }
}
