use std::env;
use std::path::{Path, PathBuf};

use anyhow::{bail, Result};

use script_2_script::{build_base_video, VideoClip};

fn main() -> Result<()> {
    // This example stitches multiple mp4 clips into one base video track.
    // You need to provide these files in assets/ for the demo to run.
    let clip_a = VideoClip::new(
        "assets/clip_a.mp4",
        0.0,
        6.0,
        Some(1.0),
        Some(9.0),
    )?;
    let clip_b = VideoClip::new(
        "assets/clip_b.mp4",
        4.0,
        10.0,
        Some(0.0),
        Some(8.0),
    )?;

    // Newest clip wins on overlaps; later clips are declared later.
    let clips = vec![clip_a, clip_b];

    // Render settings define the project resolution and fps for normalization.
    let args = RenderArgs::from_env()?;
    let output_path = args.resolve_output("m5_video_clips")?;
    std::fs::create_dir_all(output_path.parent().unwrap_or(Path::new(".")))?;

    // Temp directory holds normalized/trimmed segments for ffmpeg concat.
    let temp_dir = output_path.with_file_name("temp_video");
    build_base_video(
        &clips,
        args.width,
        args.height,
        args.fps,
        &output_path,
        &temp_dir,
        args.keep_temp,
    )?;

    Ok(())
}

struct RenderArgs {
    width: u32,
    height: u32,
    fps: u32,
    output: Option<PathBuf>,
    keep_temp: bool,
}

impl RenderArgs {
    fn from_env() -> Result<Self> {
        let mut width = 800;
        let mut height = 600;
        let mut fps = 30;
        let mut output = None;
        let mut keep_temp = false;

        let mut args = env::args().skip(1);
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--width" => {
                    let value = args
                        .next()
                        .ok_or_else(|| anyhow::anyhow!("--width requires a value"))?;
                    width = value.parse::<u32>()?;
                }
                "--height" => {
                    let value = args
                        .next()
                        .ok_or_else(|| anyhow::anyhow!("--height requires a value"))?;
                    height = value.parse::<u32>()?;
                }
                "--fps" => {
                    let value = args
                        .next()
                        .ok_or_else(|| anyhow::anyhow!("--fps requires a value"))?;
                    fps = value.parse::<u32>()?;
                }
                "--output" => {
                    let value =
                        args.next().ok_or_else(|| anyhow::anyhow!("--output requires a value"))?;
                    output = Some(PathBuf::from(value));
                }
                "--keep-temp" => keep_temp = true,
                other => bail!("unknown argument: {other}"),
            }
        }

        Ok(Self {
            width,
            height,
            fps,
            output,
            keep_temp,
        })
    }

    fn resolve_output(&self, default_stem: &str) -> Result<PathBuf> {
        if let Some(path) = &self.output {
            return Ok(path.clone());
        }
        Ok(PathBuf::from(format!("output/{default_stem}.mp4")))
    }
}
