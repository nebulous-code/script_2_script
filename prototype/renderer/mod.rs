pub mod bouncing_ball;

use anyhow::{bail, Context, Result};
use raylib::consts::PixelFormat;
use raylib::prelude::*;

use crate::audio::AudioEngine;
use crate::config::Config;
use crate::renderer::bouncing_ball::BouncingBall;

pub struct RenderedFrame {
    pub rgba: Vec<u8>,
    pub bounced: bool,
}

pub struct BouncingBallRenderer {
    rl: RaylibHandle,
    thread: RaylibThread,
    render_texture: RenderTexture2D,
    state: BouncingBall,
    audio: Option<AudioEngine>,
    width: u32,
    height: u32,
    fps: u32,
    preview_realtime: bool,
    bg: Color,
}

impl BouncingBallRenderer {
    pub fn new(config: &Config) -> Result<Self> {
        let (rl, thread) = raylib::init()
            .size(config.width as i32, config.height as i32)
            .title("Raylib → FFmpeg Renderer")
            .build();

        let mut rl = rl;
        if config.preview_realtime && config.fps > 0 {
            rl.set_target_fps(config.fps);
        }

        let render_texture = rl
            .load_render_texture(&thread, config.width, config.height)
            .context("failed to create render texture")?;

        let audio = AudioEngine::new(config)?;

        Ok(Self {
            rl,
            thread,
            render_texture,
            state: BouncingBall::new(config.width as f32, config.height as f32),
            audio,
            width: config.width,
            height: config.height,
            fps: config.fps,
            preview_realtime: config.preview_realtime,
            bg: Color::new(0x18, 0x18, 0x18, 255),
        })
    }

    pub fn window_should_close(&mut self) -> bool {
        self.rl.window_should_close()
    }

    pub fn render_frame(&mut self, frame_index: u32, total_frames: u32) -> Result<RenderedFrame> {
        let dt = if self.fps > 0 { 1.0 / self.fps as f32 } else { 0.0 };
        let bounce = self.state.step(dt);

        let t_norm = if total_frames > 0 {
            frame_index as f32 / total_frames as f32
        } else {
            0.0
        };

        {
            let mut d = self
                .rl
                .begin_texture_mode(&self.thread, self.render_texture.as_mut());
            d.clear_background(self.bg);
            let color = self.state.color_at(t_norm);
            d.draw_circle_v(self.state.position, self.state.radius, color);
        }

        if self.preview_realtime {
            let mut d = self.rl.begin_drawing(&self.thread);
            d.clear_background(self.bg);
            let color = self.state.color_at(t_norm);
            d.draw_circle_v(self.state.position, self.state.radius, color);
        }

        if let Some(audio) = self.audio.as_mut() {
            audio.update();
            if bounce.bounced {
                audio.play_bounce();
            }
        }

        let rgba = capture_rgba(&self.render_texture, self.width, self.height)?;

        Ok(RenderedFrame {
            rgba,
            bounced: bounce.bounced,
        })
    }
}

fn capture_rgba(
    render_texture: &RenderTexture2D,
    expected_w: u32,
    expected_h: u32,
) -> Result<Vec<u8>> {
    let mut image = unsafe { raylib::ffi::LoadImageFromTexture(*render_texture.texture().as_ref()) };

    let result = (|| {
        if image.data.is_null() {
            bail!("raylib returned null image data");
        }

        if image.format != PixelFormat::PIXELFORMAT_UNCOMPRESSED_R8G8B8A8 as i32 {
            unsafe {
                raylib::ffi::ImageFormat(
                    &mut image,
                    PixelFormat::PIXELFORMAT_UNCOMPRESSED_R8G8B8A8 as i32,
                );
            }
        }

        if image.data.is_null() {
            bail!("image data was null after format conversion");
        }

        let width = image.width as u32;
        let height = image.height as u32;
        if width != expected_w || height != expected_h {
            bail!(
                "capture size mismatch: got {}x{}, expected {}x{}",
                width,
                height,
                expected_w,
                expected_h
            );
        }

        let len = (width * height * 4) as usize;
        let bytes = unsafe { std::slice::from_raw_parts(image.data as *const u8, len) };
        Ok(bytes.to_vec())
    })();

    unsafe {
        raylib::ffi::UnloadImage(image);
    }

    result
}
