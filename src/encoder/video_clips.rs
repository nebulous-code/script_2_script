use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use anyhow::{bail, Context, Result};

use crate::video::{resolve_segments, VideoClip};

#[derive(Debug, Clone, Copy)]
pub struct VideoMetadata {
    pub width: u32,
    pub height: u32,
    pub fps: f32,
}

pub fn ffprobe_metadata(path: &Path) -> Result<VideoMetadata> {
    let output = Command::new("ffprobe")
        .arg("-v")
        .arg("error")
        .arg("-select_streams")
        .arg("v:0")
        .arg("-show_entries")
        .arg("stream=width,height,r_frame_rate")
        .arg("-of")
        .arg("default=nw=1")
        .arg(path)
        .output()
        .context("failed to run ffprobe")?;

    if !output.status.success() {
        bail!(
            "ffprobe failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut width = None;
    let mut height = None;
    let mut fps = None;
    for line in stdout.lines() {
        if let Some(value) = line.strip_prefix("width=") {
            width = value.parse::<u32>().ok();
        } else if let Some(value) = line.strip_prefix("height=") {
            height = value.parse::<u32>().ok();
        } else if let Some(value) = line.strip_prefix("r_frame_rate=") {
            fps = parse_rate(value);
        }
    }

    Ok(VideoMetadata {
        width: width.context("ffprobe missing width")?,
        height: height.context("ffprobe missing height")?,
        fps: fps.context("ffprobe missing fps")?,
    })
}

pub fn normalize_if_needed(
    input: &Path,
    meta: VideoMetadata,
    target_width: u32,
    target_height: u32,
    target_fps: u32,
    temp_dir: &Path,
) -> Result<PathBuf> {
    let fps_match = (meta.fps - target_fps as f32).abs() < 0.01;
    if meta.width == target_width && meta.height == target_height && fps_match {
        return Ok(input.to_path_buf());
    }

    std::fs::create_dir_all(temp_dir).context("failed to create temp dir")?;
    let output = temp_dir.join(normalized_name(input));

    let status = Command::new("ffmpeg")
        .arg("-y")
        .arg("-loglevel")
        .arg("error")
        .arg("-i")
        .arg(input)
        .arg("-vf")
        .arg(format!("scale={}x{}", target_width, target_height))
        .arg("-r")
        .arg(target_fps.to_string())
        .arg("-an")
        .arg("-c:v")
        .arg("libx264")
        .arg("-pix_fmt")
        .arg("yuv420p")
        .arg(&output)
        .stderr(Stdio::piped())
        .status()
        .context("failed to run ffmpeg normalize")?;

    if !status.success() {
        bail!("ffmpeg normalize failed for {}", input.display());
    }

    Ok(output)
}

pub fn build_base_video(
    clips: &[VideoClip],
    target_width: u32,
    target_height: u32,
    target_fps: u32,
    output_path: &Path,
    temp_dir: &Path,
    keep_temp: bool,
) -> Result<()> {
    let segments = resolve_segments(clips);
    if segments.is_empty() {
        bail!("no video segments to render");
    }

    std::fs::create_dir_all(temp_dir).context("failed to create temp dir")?;
    let mut normalized_cache = Vec::new();

    let mut segment_paths = Vec::new();
    for (seg_index, segment) in segments.iter().enumerate() {
        let clip = &clips[segment.clip_index];
        let meta = ffprobe_metadata(&clip.path)?;
        let normalized = normalize_if_needed(
            &clip.path,
            meta,
            target_width,
            target_height,
            target_fps,
            temp_dir,
        )?;

        if !normalized_cache.contains(&normalized) && normalized != clip.path {
            normalized_cache.push(normalized.clone());
        }

        let seg_duration = segment.timeline_end - segment.timeline_start;
        let seg_output = temp_dir.join(format!("segment_{:03}.mp4", seg_index));

        let mut cmd = Command::new("ffmpeg");
        cmd.arg("-y")
            .arg("-loglevel")
            .arg("error")
            .arg("-ss")
            .arg(format!("{:.3}", segment.source_start))
            .arg("-t")
            .arg(format!("{:.3}", seg_duration))
            .arg("-i")
            .arg(&normalized)
            .arg("-an");

        if normalized == clip.path && meta.width == target_width && meta.height == target_height {
            cmd.arg("-c")
                .arg("copy")
                .arg(&seg_output);
        } else {
            cmd.arg("-vf")
                .arg(format!("scale={}x{}", target_width, target_height))
                .arg("-r")
                .arg(target_fps.to_string())
                .arg("-c:v")
                .arg("libx264")
                .arg("-pix_fmt")
                .arg("yuv420p")
                .arg(&seg_output);
        }

        let status = cmd.status().context("failed to run ffmpeg segment")?;
        if !status.success() {
            bail!("ffmpeg segment render failed");
        }

        segment_paths.push(seg_output);
    }

    let list_path = temp_dir.join("concat_list.txt");
    write_concat_list(&list_path, &segment_paths)?;

    let status = Command::new("ffmpeg")
        .arg("-y")
        .arg("-loglevel")
        .arg("error")
        .arg("-f")
        .arg("concat")
        .arg("-safe")
        .arg("0")
        .arg("-i")
        .arg(&list_path)
        .arg("-c")
        .arg("copy")
        .arg(output_path)
        .status()
        .context("failed to run ffmpeg concat")?;

    if !status.success() {
        bail!("ffmpeg concat failed");
    }

    if !keep_temp {
        let _ = std::fs::remove_file(&list_path);
        for path in &segment_paths {
            let _ = std::fs::remove_file(path);
        }
        for path in normalized_cache {
            let _ = std::fs::remove_file(path);
        }
    }

    Ok(())
}

fn write_concat_list(list_path: &Path, segments: &[PathBuf]) -> Result<()> {
    let mut file = File::create(list_path).context("failed to create concat list")?;
    for seg in segments {
        let abs = seg
            .canonicalize()
            .unwrap_or_else(|_| seg.to_path_buf());
        writeln!(file, "file '{}'", abs.display())?;
    }
    Ok(())
}

fn normalized_name(path: &Path) -> String {
    let stem = path
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "clip".to_string());
    format!("{stem}_normalized.mp4")
}

fn parse_rate(rate: &str) -> Option<f32> {
    if let Some((num, den)) = rate.split_once('/') {
        let num: f32 = num.parse().ok()?;
        let den: f32 = den.parse().ok()?;
        if den == 0.0 {
            None
        } else {
            Some(num / den)
        }
    } else {
        rate.parse::<f32>().ok()
    }
}
