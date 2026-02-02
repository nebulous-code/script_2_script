use std::ffi::CString;
use std::path::Path;

use anyhow::{bail, Context, Result};

pub struct AudioEngine {
    music: raylib::ffi::Music,
    bounce: raylib::ffi::Sound,
}

impl AudioEngine {
    pub fn new(background_path: impl AsRef<Path>, bounce_path: impl AsRef<Path>) -> Result<Self> {
        let background_path = background_path.as_ref();
        let bounce_path = bounce_path.as_ref();

        if !background_path.exists() {
            bail!("missing background music: {}", background_path.display());
        }
        if !bounce_path.exists() {
            bail!("missing bounce sound: {}", bounce_path.display());
        }

        unsafe {
            if !raylib::ffi::IsAudioDeviceReady() {
                raylib::ffi::InitAudioDevice();
            }
        }

        if unsafe { !raylib::ffi::IsAudioDeviceReady() } {
            bail!("raylib audio device not ready");
        }

        let c_music = CString::new(background_path.to_string_lossy().as_bytes())
            .context("background music path contains null byte")?;
        let c_sound = CString::new(bounce_path.to_string_lossy().as_bytes())
            .context("bounce sound path contains null byte")?;

        let mut music = unsafe { raylib::ffi::LoadMusicStream(c_music.as_ptr()) };
        if music.stream.buffer.is_null() {
            unsafe { raylib::ffi::CloseAudioDevice() };
            bail!("failed to load background music: {}", background_path.display());
        }

        let bounce = unsafe { raylib::ffi::LoadSound(c_sound.as_ptr()) };
        if bounce.stream.buffer.is_null() {
            unsafe {
                raylib::ffi::UnloadMusicStream(music);
                raylib::ffi::CloseAudioDevice();
            }
            bail!("failed to load bounce sound: {}", bounce_path.display());
        }

        music.looping = true;
        unsafe {
            raylib::ffi::SetMusicVolume(music, 0.25);
            raylib::ffi::SetSoundVolume(bounce, 0.7);
            raylib::ffi::PlayMusicStream(music);
        }

        Ok(Self { music, bounce })
    }

    pub fn start_background(&mut self, looped: bool) {
        self.music.looping = looped;
        unsafe {
            raylib::ffi::PlayMusicStream(self.music);
        }
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
