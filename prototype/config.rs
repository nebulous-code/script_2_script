use std::path::PathBuf;

use chrono::Local;

#[derive(Debug, Clone)]
pub struct Config {
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    pub duration_secs: u32,
    pub output_dir: PathBuf,
    pub output_name: String,
    pub enable_audio: bool,
    pub preview_realtime: bool,
    pub background_music_path: PathBuf,
    pub bounce_sound_path: PathBuf,
}

impl Config {
    pub fn total_frames(&self) -> u32 {
        self.fps.saturating_mul(self.duration_secs)
    }

    pub fn output_path(&self) -> PathBuf {
        self.output_dir.join(&self.output_name)
    }

    pub fn validate_assets(&self) -> Vec<String> {
        let mut warnings = Vec::new();
        if !self.enable_audio {
            return warnings;
        }

        if !self.background_music_path.exists() {
            warnings.push(format!(
                "missing background music: {}",
                self.background_music_path.display()
            ));
        }

        if !self.bounce_sound_path.exists() {
            warnings.push(format!(
                "missing bounce sound: {}",
                self.bounce_sound_path.display()
            ));
        }

        warnings
    }
}

impl Default for Config {
    fn default() -> Self {
        let timestamp = Local::now().format("%Y%m%d_%H%M%S").to_string();
        Self {
            width: 800,
            height: 600,
            fps: 30,
            duration_secs: 60,
            output_dir: PathBuf::from("output"),
            output_name: format!("render_{timestamp}.mp4"),
            enable_audio: true,
            preview_realtime: true,
            background_music_path: PathBuf::from("input/background.mp3"),
            bounce_sound_path: PathBuf::from("input/border.ogg"),
        }
    }
}
