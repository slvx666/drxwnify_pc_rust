use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default)]
    pub theme_day: bool,
    #[serde(default = "default_true")]
    pub background_glyphs: bool,
    #[serde(default = "default_true")]
    pub background_motion: bool,
    #[serde(default = "default_true")]
    pub smooth_animations: bool,
    #[serde(default = "default_scale")]
    pub ui_scale: f32,
    #[serde(default = "default_lang")]
    pub language: String,
    #[serde(default)]
    pub hotkeys: HashMap<String, String>,
    #[serde(default = "default_true")]
    pub global_hotkeys: bool,
    #[serde(default = "default_quality")]
    pub quality: String,
    #[serde(default = "default_pick_mode")]
    pub pick_mode: String,
    #[serde(default)]
    pub crossfade_sec: u32,
    #[serde(default = "default_true")]
    pub normalize: bool,
    #[serde(default)]
    pub skip_silence: bool,
    #[serde(default = "default_true")]
    pub endless_queue: bool,
    #[serde(default)]
    pub output_device: Option<String>,
    #[serde(default = "default_volume")]
    pub volume: f32,
    #[serde(default = "default_last_volume")]
    pub last_volume: f32,
    #[serde(default)]
    pub balance: f32,
    #[serde(default = "default_sources")]
    pub sources: Vec<String>,
    #[serde(default)]
    pub disabled_sources: Vec<String>,
    #[serde(default = "default_catalog")]
    pub catalog: String,
    #[serde(default = "default_true")]
    pub spotify_sync_likes: bool,
    #[serde(default = "default_region")]
    pub content_region: String,
    #[serde(default = "default_lyrics_providers")]
    pub lyrics_providers: Vec<String>,
    #[serde(default)]
    pub translate_lyrics: bool,
    #[serde(default)]
    pub download_dir: Option<String>,
    #[serde(default = "default_download_format")]
    pub download_format: String,
    #[serde(default = "default_cover_cache_mb")]
    pub cover_cache_mb: u32,
    #[serde(default = "default_true")]
    pub share_listening: bool,
    #[serde(default)]
    pub pause_history: bool,
    #[serde(default = "default_left_width")]
    pub left_width: f32,
    #[serde(default)]
    pub left_collapsed: bool,
    #[serde(default = "default_right_width")]
    pub right_width: f32,
    #[serde(default)]
    pub right_hidden: bool,
    #[serde(default = "default_window_w")]
    pub window_w: i32,
    #[serde(default = "default_window_h")]
    pub window_h: i32,
    #[serde(default)]
    pub window_maximized: bool,
}

fn default_theme() -> String { "tategaki".to_string() }
fn default_true() -> bool { true }
fn default_scale() -> f32 { 1.0 }
fn default_lang() -> String { "ru".to_string() }
fn default_quality() -> String { "max".to_string() }
fn default_pick_mode() -> String { "quality".to_string() }
fn default_volume() -> f32 { 0.8 }
fn default_last_volume() -> f32 { 0.5 }
fn default_sources() -> Vec<String> {
    vec![
        "soundcloud".to_string(),
        "youtube".to_string(),
        "vk".to_string(),
        "lossless".to_string(),
        "bandcamp".to_string(),
        "audius".to_string(),
    ]
}
fn default_catalog() -> String { "spotify".to_string() }
fn default_region() -> String { "RU".to_string() }
fn default_lyrics_providers() -> Vec<String> {
    vec!["lrclib".to_string(), "betterlyrics".to_string(), "kugou".to_string()]
}
fn default_download_format() -> String { "original".to_string() }
fn default_cover_cache_mb() -> u32 { 300 }
fn default_left_width() -> f32 { 270.0 }
fn default_right_width() -> f32 { 310.0 }
fn default_window_w() -> i32 { 1360 }
fn default_window_h() -> i32 { 860 }

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            theme: default_theme(),
            theme_day: false,
            background_glyphs: true,
            background_motion: true,
            smooth_animations: true,
            ui_scale: default_scale(),
            language: default_lang(),
            hotkeys: HashMap::new(),
            global_hotkeys: true,
            quality: default_quality(),
            pick_mode: default_pick_mode(),
            crossfade_sec: 0,
            normalize: true,
            skip_silence: false,
            endless_queue: true,
            output_device: None,
            volume: default_volume(),
            last_volume: default_last_volume(),
            balance: 0.0,
            sources: default_sources(),
            disabled_sources: Vec::new(),
            catalog: default_catalog(),
            spotify_sync_likes: true,
            content_region: default_region(),
            lyrics_providers: default_lyrics_providers(),
            translate_lyrics: false,
            download_dir: None,
            download_format: default_download_format(),
            cover_cache_mb: default_cover_cache_mb(),
            share_listening: true,
            pause_history: false,
            left_width: default_left_width(),
            left_collapsed: false,
            right_width: default_right_width(),
            right_hidden: false,
            window_w: default_window_w(),
            window_h: default_window_h(),
            window_maximized: false,
        }
    }
}
