use std::env;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};

use raylib_playground::{
    AnimatedTransform, Clip, Color, FfmpegVideoEncoder, Layer, Object, RaylibRender, Shape,
    Timeline, Transform, Vec2,
};

fn main() -> Result<()> {
    let mut timeline = Timeline::new(6.0, 30)?;
    let args = RenderArgs::from_env(timeline.duration)?;

    let mut background = Layer::new("background");
    background.add_clip(Clip::new(
        0.0,
        6.0,
        Object::Shape(Shape::Rect {
            width: 760.0,
            height: 460.0,
            color: Color::rgb(18, 18, 22),
        }),
        AnimatedTransform::constant(Transform::default()),
        timeline.duration,
    )?);

    let mut motion = Layer::new("motion");
    motion.add_clip(Clip::new(
        0.0,
        6.0,
        Object::Shape(Shape::Circle {
            radius: 70.0,
            color: Color::rgb(235, 90, 90),
        }),
        AnimatedTransform::constant(Transform {
            pos: Vec2 { x: 0.0, y: 0.0 },
            ..Transform::default()
        }),
        timeline.duration,
    )?);

    timeline.add_layer(background);
    timeline.add_layer(motion);

    let output_path = args.resolve_output("m3_render_video")?;
    let temp_path = if args.keep_temp {
        temp_output_path(&output_path)
    } else {
        output_path.clone()
    };

    let mut renderer = RaylibRender::new(800, 600, Color::rgb(16, 16, 20))?;
    let mut encoder = FfmpegVideoEncoder::start(800, 600, timeline.fps, &temp_path)?;

    renderer.render_timeline_rgba(&timeline, args.start_time, args.end_time, |_t, rgba| {
        encoder.write_frame(rgba)
    })?;

    encoder.finish()?;

    if args.keep_temp {
        std::fs::copy(&temp_path, &output_path).with_context(|| {
            format!("failed to copy temp output to {}", output_path.display())
        })?;
    }

    Ok(())
}

struct RenderArgs {
    start_time: f32,
    end_time: f32,
    output: Option<PathBuf>,
    keep_temp: bool,
}

impl RenderArgs {
    fn from_env(duration: f32) -> Result<Self> {
        let mut start_time = 0.0;
        let mut end_time = duration;
        let mut output = None;
        let mut keep_temp = false;

        let mut args = env::args().skip(1);
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--render" => {}
                "--start_time" => {
                    let value = args
                        .next()
                        .ok_or_else(|| anyhow::anyhow!("--start_time requires a value"))?;
                    start_time = value.parse::<f32>()?;
                }
                "--end_time" => {
                    let value =
                        args.next().ok_or_else(|| anyhow::anyhow!("--end_time requires a value"))?;
                    end_time = value.parse::<f32>()?;
                }
                "--output" => {
                    let value =
                        args.next().ok_or_else(|| anyhow::anyhow!("--output requires a value"))?;
                    output = Some(PathBuf::from(value));
                }
                "--keep-temp" => {
                    keep_temp = true;
                }
                other => bail!("unknown argument: {other}"),
            }
        }

        if start_time < 0.0 || end_time <= start_time || end_time > duration {
            bail!("start/end time must satisfy 0 <= start < end <= duration");
        }

        Ok(Self {
            start_time,
            end_time,
            output,
            keep_temp,
        })
    }

    fn resolve_output(&self, default_stem: &str) -> Result<PathBuf> {
        if let Some(path) = &self.output {
            return Ok(path.clone());
        }

        Ok(PathBuf::from(format!("{default_stem}.mp4")))
    }
}

fn temp_output_path(output_path: &Path) -> PathBuf {
    let stem = output_path
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "render".to_string());
    let ext = output_path.extension().map(|e| e.to_string_lossy());
    match ext {
        Some(ext) => output_path.with_file_name(format!("{stem}.video.{ext}")),
        None => output_path.with_file_name(format!("{stem}.video.mp4")),
    }
}
