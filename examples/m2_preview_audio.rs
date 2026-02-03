use std::env;
use std::path::{Path, PathBuf};

use anyhow::{bail, Result};

use script_2_script::{
    AnimatedTransform, AudioEngine, Clip, Color, Easing, FfmpegVideoEncoder, ImageObject, Keyframe,
    Layer, Object, RaylibPreview, RaylibRender, Shape, Timeline, Track, Transform, Vec2,
};

fn main() -> Result<()> {
    // Timeline: this is the scene we will preview with sound.
    let mut timeline = Timeline::new(6.0, 30)?;
    let args = RenderArgs::from_env(timeline.duration)?;

    // Background layer so the scene has a visible backdrop.
    let mut background = Layer::new("background");
    background.add_clip(Clip::new(
        0.0,
        6.0,
        Object::Shape(Shape::Rect {
            width: 760.0,
            height: 460.0,
            color: Color::rgb(18, 18, 22),
        }),
        AnimatedTransform::default(),
        timeline.duration,
    )?);

    // Animated layer to give the preview something moving.
    let mut motion = Layer::new("motion");
    let circle_transform = AnimatedTransform {
        position: Track::new(vec![
            Keyframe::new(0.0, Vec2 { x: -280.0, y: -40.0 }, Easing::EaseInOutQuad),
            Keyframe::new(4.0, Vec2 { x: 280.0, y: 40.0 }, Easing::Linear),
        ])?,
        scale: Track::from_constant(Vec2 { x: 1.0, y: 1.0 }),
        rotation: Track::from_constant(0.0),
        opacity: Track::from_constant(1.0),
    };

    motion.add_clip(Clip::new(
        0.0,
        5.0,
        Object::Shape(Shape::Circle {
            radius: 60.0,
            color: Color::rgb(235, 90, 90),
        }),
        circle_transform,
        timeline.duration,
    )?);

    motion.add_clip(Clip::new(
        0.5,
        5.5,
        Object::Image(ImageObject::new("assets/logo.png")),
        AnimatedTransform::constant(Transform {
            pos: Vec2 { x: 0.0, y: -40.0 },
            scale: Vec2 { x: 2.0, y: 2.0 },
            ..Transform::default()
        }),
        timeline.duration,
    )?);

    timeline.add_layer(background);
    timeline.add_layer(motion);

    if args.render {
        let output_path = args.resolve_output("m2_preview_audio")?;
        std::fs::create_dir_all(output_path.parent().unwrap_or(Path::new(".")))?;
        let mut renderer = RaylibRender::new(800, 600, Color::rgb(16, 16, 20))?;
        let mut encoder =
            FfmpegVideoEncoder::start(800, 600, timeline.fps, &output_path)?;
        renderer.render_timeline_rgba(&timeline, args.start_time, args.end_time, |_t, rgba| {
            encoder.write_frame(rgba)
        })?;
        encoder.finish()
    } else {
        // Preview window uses fixed-dt sampling; audio is updated each frame.
        let preview = RaylibPreview::new(800, 600, Color::rgb(16, 16, 20));
        let mut audio = AudioEngine::new("assets/background.mp3", "assets/border.ogg")?;
        audio.start_background(true);

        // Play a bounce sound every time the integer second increases.
        let mut last_second = args.start_time.floor() as i32;
        preview.run_with(&timeline, args.start_time, args.end_time, |t| {
            audio.update();
            let current = t.floor() as i32;
            if current > last_second {
                audio.play_bounce();
                last_second = current;
            }
            Ok(())
        })
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
                    let value = args.next().ok_or_else(|| {
                        anyhow::anyhow!("--start_time requires a value in seconds")
                    })?;
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
