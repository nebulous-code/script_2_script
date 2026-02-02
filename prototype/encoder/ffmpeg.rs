use std::io::Write;
use std::path::Path;
use std::process::{Child, ChildStdin, Command, Stdio};

use anyhow::{bail, Context, Result};

use crate::config::Config;

pub struct FfmpegEncoder {
    child: Child,
    stdin: Option<ChildStdin>,
    width: u32,
    height: u32,
    fps: u32,
}

impl FfmpegEncoder {
    pub fn start(config: &Config) -> Result<Self> {
        let output_path = config.output_path();
        Self::start_with_output(config, &output_path)
    }

    pub fn start_with_output(config: &Config, output_path: &Path) -> Result<Self> {
        std::fs::create_dir_all(&config.output_dir)
            .context("failed to create output directory")?;

        let mut cmd = Command::new("ffmpeg");
        cmd.arg("-y")
            .arg("-loglevel")
            .arg("error")
            .arg("-f")
            .arg("rawvideo")
            .arg("-pix_fmt")
            .arg("rgba")
            .arg("-s")
            .arg(format!("{}x{}", config.width, config.height))
            .arg("-r")
            .arg(config.fps.to_string())
            .arg("-i")
            .arg("-")
            .arg("-vf")
            .arg("vflip")
            .arg("-c:v")
            .arg("libx264")
            .arg("-pix_fmt")
            .arg("yuv420p")
            .arg("-crf")
            .arg("18")
            .arg(output_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::inherit());

        let mut child = cmd
            .spawn()
            .context("failed to spawn ffmpeg (is it on PATH?)")?;
        let stdin = child
            .stdin
            .take()
            .context("failed to open ffmpeg stdin")?;

        Ok(Self {
            child,
            stdin: Some(stdin),
            width: config.width,
            height: config.height,
            fps: config.fps,
        })
    }

    pub fn write_frame(&mut self, frame: &[u8]) -> Result<()> {
        let expected = (self.width * self.height * 4) as usize;
        if frame.len() != expected {
            bail!(
                "frame size mismatch: got {}, expected {}",
                frame.len(),
                expected
            );
        }

        let stdin = self
            .stdin
            .as_mut()
            .context("ffmpeg stdin already closed")?;
        stdin.write_all(frame).context("failed to write frame")?;
        Ok(())
    }

    pub fn finish(mut self) -> Result<()> {
        if let Some(mut stdin) = self.stdin.take() {
            let _ = stdin.flush();
        }

        let status = self.child.wait().context("failed to wait for ffmpeg")?;
        if !status.success() {
            bail!("ffmpeg exited with status {}", status);
        }

        Ok(())
    }
}

pub fn mux_audio(
    video_path: &Path,
    background_path: &Path,
    bounce_path: &Path,
    bounce_times_ms: &[i64],
    output_path: &Path,
) -> Result<()> {
    let mut cmd = Command::new("ffmpeg");
    cmd.arg("-y")
        .arg("-loglevel")
        .arg("error")
        .arg("-i")
        .arg(video_path)
        .arg("-stream_loop")
        .arg("-1")
        .arg("-i")
        .arg(background_path);

    let has_bounces = !bounce_times_ms.is_empty();
    if has_bounces {
        cmd.arg("-i").arg(bounce_path);
        let filter = build_bounce_filter(bounce_times_ms);
        cmd.arg("-filter_complex").arg(filter)
            .arg("-map")
            .arg("0:v:0")
            .arg("-map")
            .arg("[aout]");
    } else {
        cmd.arg("-map").arg("0:v:0").arg("-map").arg("1:a:0");
    }

    cmd.arg("-c:v")
        .arg("copy")
        .arg("-c:a")
        .arg("aac")
        .arg("-shortest")
        .arg(output_path)
        .stdout(Stdio::null())
        .stderr(Stdio::inherit());

    let status = cmd
        .spawn()
        .context("failed to spawn ffmpeg for audio mux")?
        .wait()
        .context("failed to wait for ffmpeg audio mux")?;

    if !status.success() {
        bail!("ffmpeg audio mux exited with status {}", status);
    }

    Ok(())
}

fn build_bounce_filter(bounce_times_ms: &[i64]) -> String {
    let split_count = bounce_times_ms.len();
    let mut filter = String::new();

    filter.push_str(&format!("[2:a]asplit={}", split_count));
    for i in 0..split_count {
        filter.push_str(&format!("[b{}]", i));
    }
    filter.push(';');

    for (i, delay) in bounce_times_ms.iter().enumerate() {
        filter.push_str(&format!("[b{0}]adelay={1}|{1}[bd{0}];", i, delay));
    }

    filter.push_str("[1:a]");
    for i in 0..split_count {
        filter.push_str(&format!("[bd{}]", i));
    }
    filter.push_str(&format!("amix=inputs={}:normalize=0[aout]", split_count + 1));

    filter
}
