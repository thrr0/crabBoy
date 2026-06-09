use std::fs;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub keybinds: KeyBinds,
    pub scale: u8,
    pub palette: [u32; 4],
    pub filter: String,
    pub keep_aspect_ratio: bool,
    pub volume: f32,
    pub audio_device: String,
    pub channel_1: bool,
    pub channel_2: bool,
    pub channel_3: bool,
    pub channel_4: bool,
    pub turbo_speed: f32,
    pub skip_boot_rom: bool,
    pub language: String,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            keybinds: KeyBinds {
                up: "Up".to_string(),
                down: "Down".to_string(),
                left: "Left".to_string(),
                right: "Right".to_string(),
                a: "Z".to_string(),
                b: "X".to_string(),
                select: "Backspace".to_string(),
                start: "Enter".to_string(),
                fullscreen: "F11".to_string(),
                fast_forward: "Space".to_string(),
                volume_up: "Key1".to_string(),
                volume_down: "Key2".to_string(),
            },
            scale: 3,
            palette: [0xC4CFA1, 0x8B956D, 0x4D533C, 0x1F1F1F],
            filter: "none".to_string(),
            keep_aspect_ratio: true,
            volume: 0.5,
            audio_device: "default".to_string(),
            channel_1: true,
            channel_2: true,
            channel_3: true,
            channel_4: true,
            turbo_speed: 4.0,
            skip_boot_rom: false,
            language: "English".to_string(),
        }
    }
}

impl Config {
    pub fn load(path: &str) -> Config {
        fs::read_to_string(path)
            .ok()
            .and_then(|content| toml::from_str(&content).ok())
            .unwrap_or_default()
    }

    pub fn save(&self, path: &str) -> std::io::Result<()> {
        let toml_string =
            toml::to_string(self).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

        fs::write(path, toml_string)?;

        Ok(())
    }
}
#[derive(Serialize, Deserialize)]
struct KeyBinds {
    up: String,
    down: String,
    left: String,
    right: String,
    a: String,
    b: String,
    select: String,
    start: String,
    fullscreen: String,
    fast_forward: String,
    volume_up: String,
    volume_down: String,
}
