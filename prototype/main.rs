mod audio;
mod config;
mod encoder;
mod renderer;

use std::ffi::OsString;
use std::path::{Path, PathBuf};

use anyhow::{bail, Result};

use config::Config;
use encoder::{mux_audio, FfmpegEncoder};
use renderer::BouncingBallRenderer;

fn main() -> Result<()> {
    let config = Config::default();
    let total_frames = config.total_frames();
    if total_frames == 0 {
        bail!("fps * duration must be > 0");
    }

    let warnings = config.validate_assets();
    let audio_ok = config.enable_audio && warnings.is_empty();
    if config.enable_audio && !warnings.is_empty() {
        for warning in warnings {
            eprintln!("audio disabled: {warning}");
        }
    }

    let output_path = config.output_path();
    let video_path = if audio_ok {
        temp_video_path(&output_path)
    } else {
        output_path.clone()
    };

    let mut renderer = BouncingBallRenderer::new(&config)?;
    let mut encoder = FfmpegEncoder::start_with_output(&config, &video_path)?;

    let mut bounce_times_ms = Vec::new();
    for frame_index in 0..total_frames {
        if renderer.window_should_close() {
            break;
        }
        let rendered = renderer.render_frame(frame_index, total_frames)?;
        if rendered.bounced && config.fps > 0 {
            let ms = ((frame_index as f64) * 1000.0 / config.fps as f64).round() as i64;
            bounce_times_ms.push(ms);
        }
        encoder.write_frame(&rendered.rgba)?;
    }

    encoder.finish()?;

    if audio_ok {
        mux_audio(
            &video_path,
            &config.background_music_path,
            &config.bounce_sound_path,
            &bounce_times_ms,
            &output_path,
        )?;
        let _ = std::fs::remove_file(&video_path);
    }

    Ok(())
}

fn temp_video_path(output_path: &Path) -> PathBuf {
    let stem = output_path
        .file_stem()
        .map(|s| s.to_os_string())
        .unwrap_or_else(|| OsString::from("render"));

    let mut name = stem;
    name.push(".video");

    if let Some(ext) = output_path.extension() {
        name.push(".");
        name.push(ext);
    }

    output_path.with_file_name(name)
}
