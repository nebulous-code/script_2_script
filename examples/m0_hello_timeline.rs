use anyhow::{bail, Result};

use std::env;
use std::path::{Path, PathBuf};

use script_2_script::{
    AnimatedTransform, Clip, Color, FfmpegVideoEncoder, ImageObject, Layer, Object, RaylibPreview,
    RaylibRender, Shape, Timeline, Transform, Vec2,
};

fn main() -> Result<()> {
    // 10-second timeline at 30 FPS (timeline time is always seconds).
    let mut timeline = Timeline::new(10.0, 30)?;
    let args = RenderArgs::from_env(timeline.duration)?;

    // Background layer: a large rectangle centered on the canvas.
    let mut background = Layer::new("background");
    background.add_clip(Clip::new(
        0.0,
        8.0,
        Object::Shape(Shape::Rect {
            width: 700.0,
            height: 420.0,
            // Note: this clip covers the screen, so it overrides the preview clear color.
            //color: Color::rgb(128, 5, 128),
            color: Color::rgba(128, 4, 128, 255),
        }),
        // Constant transform = no animation (static placement).
        AnimatedTransform::constant(Transform::default()),
        timeline.duration,
    )?);

    // Mid layer: two shapes that overlap in time to show z-ordering.
    let mut mid = Layer::new("mid");
    mid.add_clip(Clip::new(
        1.0,
        6.5,
        Object::Shape(Shape::Circle {
            radius: 80.0,
            color: Color::rgba_css(235, 101, 80, 0.7),
        }),
        AnimatedTransform::constant(Transform {
            // Graph coords: (0,0) is center; +Y is up.
            pos: Vec2 { x: -140.0, y: 60.0 },
            ..Transform::default()
        }),
        timeline.duration,
    )?);
    mid.add_clip(Clip::new(
        2.5,
        8.0,
        Object::Shape(Shape::Rect {
            width: 200.0,
            height: 120.0,
            color: Color::rgb(70, 140, 220),
        }),
        AnimatedTransform::constant(Transform {
            pos: Vec2 { x: 160.0, y: -40.0 },
            rotation: 12.0,
            ..Transform::default()
        }),
        timeline.duration,
    )?);

    // Top layer: image clip that appears later.
    let mut top = Layer::new("top");
    top.add_clip(Clip::new(
        3.0,
        8.0,
        Object::Image(ImageObject::new("assets/logo.png")),
        AnimatedTransform::constant(Transform {
            pos: Vec2 { x: 0.0, y: 0.0 },
            scale: Vec2 { x: 2.0, y: 2.0 },
            ..Transform::default()
        }),
        timeline.duration,
    )?);

    // Layer order = draw order (background → mid → top).
    timeline.add_layer(background);
    timeline.add_layer(mid);
    timeline.add_layer(top);

    // Preview window uses a clear color behind the timeline.
    let preview = RaylibPreview::new(800, 600, Color::rgb(16, 16, 20));

    if args.render {
        let output_path = args.resolve_output("m0_hello_timeline")?;
        std::fs::create_dir_all(output_path.parent().unwrap_or(Path::new(".")))?;
        let mut renderer = RaylibRender::new(800, 600, Color::rgb(16, 16, 20))?;
        let mut encoder =
            FfmpegVideoEncoder::start(800, 600, timeline.fps, &output_path)?;
        renderer.render_timeline_rgba(&timeline, args.start_time, args.end_time, |_t, rgba| {
            encoder.write_frame(rgba)
        })?;
        encoder.finish()
    } else {
        preview.run_with(&timeline, args.start_time, args.end_time, |_| Ok(()))
    }
}

struct RenderArgs {
    render: bool,
    start_time: f32,
    end_time: f32,
    output: Option<PathBuf>,
}

impl RenderArgs {
    fn from_env(duration: f32) -> Result<Self> {
        let mut render = false;
        let mut start_time = 0.0;
        let mut end_time = duration;
        let mut output = None;

        let mut args = env::args().skip(1);
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--render" => render = true,
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
                "--preview" => {}
                other => bail!("unknown argument: {other}"),
            }
        }

        if start_time < 0.0 || end_time <= start_time || end_time > duration {
            bail!("start/end time must satisfy 0 <= start < end <= duration");
        }

        Ok(Self {
            render,
            start_time,
            end_time,
            output,
        })
    }

    fn resolve_output(&self, default_stem: &str) -> Result<PathBuf> {
        if let Some(path) = &self.output {
            return Ok(path.clone());
        }
        Ok(PathBuf::from(format!("output/{default_stem}.mp4")))
    }
}
