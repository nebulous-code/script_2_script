use std::io::Read;
use std::path::Path;
use std::process::{Command, Stdio};

use anyhow::{bail, Context, Result};

use crate::audio::{MusicTrack, SfxEvent};

pub fn render_audio_wav(
    music: &MusicTrack,
    sfx: &[SfxEvent],
    duration: f32,
    output_wav: &Path,
) -> Result<()> {
    if sfx.len() > 100 {
        eprintln!("warning: high SFX event count: {}", sfx.len());
    }

    let mut cmd = Command::new("ffmpeg");
    cmd.arg("-y")
        .arg("-loglevel")
        .arg("error")
        .arg("-stream_loop")
        .arg(if music.looped { "-1" } else { "0" })
        .arg("-i")
        .arg(&music.path);

    if !sfx.is_empty() {
        let sfx_path = &sfx[0].path;
        cmd.arg("-i").arg(sfx_path);
        let filter = build_sfx_filter(sfx, music.volume);
        cmd.arg("-filter_complex")
            .arg(filter)
            .arg("-map")
            .arg("[aout]");
    } else {
        cmd.arg("-map").arg("0:a:0");
    }

    cmd.arg("-t")
        .arg(format!("{duration:.3}"))
        .arg("-c:a")
        .arg("pcm_s16le")
        .arg(output_wav)
        .stdout(Stdio::null())
        .stderr(Stdio::piped());

    let mut child = cmd
        .spawn()
        .context("failed to spawn ffmpeg for audio render")?;
    let status = child
        .wait()
        .context("failed to wait for ffmpeg audio render")?;

    if !status.success() {
        let stderr = child
            .stderr
            .take()
            .map(|mut s| {
                let mut buf = String::new();
                let _ = s.read_to_string(&mut buf);
                buf
            })
            .unwrap_or_default();
        bail!("ffmpeg audio render failed with status {}: {}", status, stderr.trim());
    }

    Ok(())
}

pub fn trim_audio(
    input_wav: &Path,
    start_time: f32,
    end_time: f32,
    output_wav: &Path,
) -> Result<()> {
    let duration = end_time - start_time;
    let mut cmd = Command::new("ffmpeg");
    cmd.arg("-y")
        .arg("-loglevel")
        .arg("error")
        .arg("-ss")
        .arg(format!("{start_time:.3}"))
        .arg("-t")
        .arg(format!("{duration:.3}"))
        .arg("-i")
        .arg(input_wav)
        .arg(output_wav)
        .stdout(Stdio::null())
        .stderr(Stdio::piped());

    let mut child = cmd
        .spawn()
        .context("failed to spawn ffmpeg for audio trim")?;
    let status = child
        .wait()
        .context("failed to wait for ffmpeg audio trim")?;

    if !status.success() {
        let stderr = child
            .stderr
            .take()
            .map(|mut s| {
                let mut buf = String::new();
                let _ = s.read_to_string(&mut buf);
                buf
            })
            .unwrap_or_default();
        bail!("ffmpeg audio trim failed with status {}: {}", status, stderr.trim());
    }

    Ok(())
}

pub fn mux_video_audio(video_path: &Path, audio_path: &Path, output_path: &Path) -> Result<()> {
    let mut cmd = Command::new("ffmpeg");
    cmd.arg("-y")
        .arg("-loglevel")
        .arg("error")
        .arg("-i")
        .arg(video_path)
        .arg("-i")
        .arg(audio_path)
        .arg("-c:v")
        .arg("copy")
        .arg("-c:a")
        .arg("aac")
        .arg("-shortest")
        .arg(output_path)
        .stdout(Stdio::null())
        .stderr(Stdio::piped());

    let mut child = cmd
        .spawn()
        .context("failed to spawn ffmpeg for mux")?;
    let status = child.wait().context("failed to wait for ffmpeg mux")?;
    if !status.success() {
        let stderr = child
            .stderr
            .take()
            .map(|mut s| {
                let mut buf = String::new();
                let _ = s.read_to_string(&mut buf);
                buf
            })
            .unwrap_or_default();
        bail!("ffmpeg mux failed with status {}: {}", status, stderr.trim());
    }

    Ok(())
}

fn build_sfx_filter(sfx: &[SfxEvent], music_volume: f32) -> String {
    let split_count = sfx.len();
    let mut filter = String::new();

    filter.push_str(&format!("[1:a]asplit={}", split_count));
    for i in 0..split_count {
        filter.push_str(&format!("[b{}]", i));
    }
    filter.push(';');

    for (i, event) in sfx.iter().enumerate() {
        let delay_ms = (event.time * 1000.0).round() as i64;
        filter.push_str(&format!(
            "[b{0}]adelay={1}|{1},volume={2}[bd{0}];",
            i, delay_ms, event.volume
        ));
    }

    filter.push_str(&format!("[0:a]volume={}[bg];", music_volume));
    filter.push_str("[bg]");
    for i in 0..split_count {
        filter.push_str(&format!("[bd{}]", i));
    }
    filter.push_str(&format!("amix=inputs={}:normalize=0[aout]", split_count + 1));

    filter
}
