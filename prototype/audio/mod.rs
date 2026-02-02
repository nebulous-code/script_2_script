use std::ffi::CString;

use anyhow::{bail, Context, Result};

use crate::config::Config;

pub struct AudioEngine {
    music: raylib::ffi::Music,
    bounce: raylib::ffi::Sound,
}

impl AudioEngine {
    pub fn new(config: &Config) -> Result<Option<Self>> {
        if !config.enable_audio {
            return Ok(None);
        }

        let warnings = config.validate_assets();
        if !warnings.is_empty() {
            for warning in warnings {
                eprintln!("audio disabled: {warning}");
            }
            return Ok(None);
        }

        unsafe {
            if !raylib::ffi::IsAudioDeviceReady() {
                raylib::ffi::InitAudioDevice();
            }
        }

        if unsafe { !raylib::ffi::IsAudioDeviceReady() } {
            bail!("raylib audio device not ready");
        }

        let music_path = config.background_music_path.to_string_lossy();
        let sound_path = config.bounce_sound_path.to_string_lossy();

        let c_music = CString::new(music_path.as_bytes())
            .context("background music path contains null byte")?;
        let c_sound = CString::new(sound_path.as_bytes())
            .context("bounce sound path contains null byte")?;

        let mut music = unsafe { raylib::ffi::LoadMusicStream(c_music.as_ptr()) };
        if music.stream.buffer.is_null() {
            unsafe {
                raylib::ffi::CloseAudioDevice();
            }
            bail!("failed to load background music: {music_path}");
        }

        let bounce = unsafe { raylib::ffi::LoadSound(c_sound.as_ptr()) };
        if bounce.stream.buffer.is_null() {
            unsafe {
                raylib::ffi::UnloadMusicStream(music);
                raylib::ffi::CloseAudioDevice();
            }
            bail!("failed to load bounce sound: {sound_path}");
        }

        music.looping = true;
        unsafe {
            raylib::ffi::SetMusicVolume(music, 0.25);
            raylib::ffi::SetSoundVolume(bounce, 0.7);
            raylib::ffi::PlayMusicStream(music);
        }

        Ok(Some(Self { music, bounce }))
    }

    pub fn update(&mut self) {
        unsafe {
            raylib::ffi::UpdateMusicStream(self.music);
        }
    }

    pub fn play_bounce(&mut self) {
        unsafe {
            raylib::ffi::PlaySound(self.bounce);
        }
    }
}

impl Drop for AudioEngine {
    fn drop(&mut self) {
        unsafe {
            raylib::ffi::UnloadSound(self.bounce);
            raylib::ffi::UnloadMusicStream(self.music);
            raylib::ffi::CloseAudioDevice();
        }
    }
}
