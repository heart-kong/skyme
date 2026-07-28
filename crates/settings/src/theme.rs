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
