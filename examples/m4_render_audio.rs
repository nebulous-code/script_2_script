use std::env;
use std::path::{Path, PathBuf};

use anyhow::{bail, Result};

use raylib_playground::{
    AnimatedTransform, Clip, Color, Easing, FfmpegVideoEncoder, Keyframe, Layer, MusicTrack,
    Object, RaylibRender, Shape, SfxEvent, Timeline, Track, Transform, Vec2,
};
use raylib_playground::{mux_video_audio, render_audio_wav, trim_audio};

fn main() -> Result<()> {
    // 25-second timeline at 30 FPS for render with audio.
    let mut timeline = Timeline::new(25.0, 30)?;
    let args = RenderArgs::from_env(timeline.duration)?;

    // Background layer: full frame rectangle as a backdrop.
    let mut background = Layer::new("background");
    background.add_clip(Clip::new(
        0.0,
        timeline.duration,
        Object::Shape(Shape::Rect {
            width: 760.0,
            height: 460.0,
            color: Color::rgb(18, 18, 22),
        }),
        AnimatedTransform::constant(Transform::default()),
        timeline.duration,
    )?);

    // Motion layer: bouncing ball, segmented by color for the RGB cycle.
    let mut motion = Layer::new("motion");
    let radius = 60.0;
    let bounds = Bounds::new(800.0, 600.0, radius);
    let mut pos = Vec2 { x: -200.0, y: 80.0 };
    let mut vel = Vec2 { x: 220.0, y: 170.0 };

    // Cycle through red → yellow → green → cyan → blue → magenta.
    let colors = [
        Color::rgb(255, 0, 0),
        Color::rgb(255, 255, 0),
        Color::rgb(0, 255, 0),
        Color::rgb(0, 255, 255),
        Color::rgb(0, 0, 255),
        Color::rgb(255, 0, 255),
    ];

    // Collect SFX events while building the bouncing path.
    let mut sfx_events = Vec::new();
    let segment = timeline.duration / colors.len() as f32;
    for (i, color) in colors.iter().enumerate() {
        let start = i as f32 * segment;
        let end = if i == colors.len() - 1 {
            timeline.duration
        } else {
            (i + 1) as f32 * segment
        };

        // Build a fixed-dt bounce path for this color segment.
        let (track, events, next_pos, next_vel) =
            build_bounce_track(start, end, timeline.fps, pos, vel, bounds)?;
        pos = next_pos;
        vel = next_vel;

        // Each bounce time becomes an SFX event.
        sfx_events.extend(events.into_iter().map(|time| SfxEvent {
            path: PathBuf::from("assets/border.ogg"),
            time,
            volume: 0.7,
        }));

        motion.add_clip(Clip::new(
            start,
            end,
            Object::Shape(Shape::Circle {
                radius,
                color: *color,
            }),
            AnimatedTransform {
                position: track,
                scale: Track::from_constant(Vec2 { x: 1.0, y: 1.0 }),
                rotation: Track::from_constant(0.0),
                opacity: Track::from_constant(1.0),
            },
            timeline.duration,
        )?);
    }

    timeline.add_layer(background);
    timeline.add_layer(motion);

    // Render video to a temp file, mix audio offline, then mux into final output.
    let output_path = args.resolve_output("m4_render_audio")?;
    std::fs::create_dir_all(output_path.parent().unwrap_or(Path::new(".")))?;
    let temp_video = temp_output_path(&output_path, "video");
    let audio_full = output_path.with_file_name("audio_full.wav");
    let audio_clip = output_path.with_file_name("audio_clip.wav");

    let mut renderer = RaylibRender::new(800, 600, Color::rgb(16, 16, 20))?;
    let mut encoder = FfmpegVideoEncoder::start(800, 600, timeline.fps, &temp_video)?;
    renderer.render_timeline_rgba(&timeline, args.start_time, args.end_time, |_t, rgba| {
        encoder.write_frame(rgba)
    })?;
    encoder.finish()?;

    // Background music track (looped to cover the full timeline).
    let music = MusicTrack {
        path: PathBuf::from("assets/background.mp3"),
        start: 0.0,
        end: timeline.duration,
        looped: true,
        volume: 0.25,
    };

    render_audio_wav(&music, &sfx_events, timeline.duration, &audio_full)?;

    // Trim audio if we rendered only a segment of the full timeline.
    let audio_for_mux = if args.start_time == 0.0 && args.end_time == timeline.duration {
        audio_full.clone()
    } else {
        trim_audio(&audio_full, args.start_time, args.end_time, &audio_clip)?;
        audio_clip.clone()
    };

    mux_video_audio(&temp_video, &audio_for_mux, &output_path)?;

    // Clean up intermediate files unless --keep-temp is set.
    if !args.keep_temp {
        let _ = std::fs::remove_file(&temp_video);
        let _ = std::fs::remove_file(&audio_full);
        let _ = std::fs::remove_file(&audio_clip);
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
        Ok(PathBuf::from(format!("output/{default_stem}.mp4")))
    }
}

#[derive(Clone, Copy)]
struct Bounds {
    min_x: f32,
    max_x: f32,
    min_y: f32,
    max_y: f32,
}

impl Bounds {
    fn new(width: f32, height: f32, radius: f32) -> Self {
        let half_w = width / 2.0;
        let half_h = height / 2.0;
        Self {
            min_x: -half_w + radius,
            max_x: half_w - radius,
            min_y: -half_h + radius,
            max_y: half_h - radius,
        }
    }
}

fn build_bounce_track(
    start: f32,
    end: f32,
    fps: u32,
    mut pos: Vec2,
    mut vel: Vec2,
    bounds: Bounds,
) -> Result<(Track<Vec2>, Vec<f32>, Vec2, Vec2)> {
    let dt = 1.0 / fps as f32;
    let frames = ((end - start) * fps as f32).floor() as u32;
    let mut keys = Vec::with_capacity(frames as usize + 1);
    let mut events = Vec::new();

    keys.push(Keyframe::new(0.0, pos, Easing::Linear));

    for i in 1..=frames {
        pos.x += vel.x * dt;
        pos.y += vel.y * dt;

        let mut bounced = false;
        if pos.x <= bounds.min_x {
            pos.x = bounds.min_x;
            vel.x = vel.x.abs();
            bounced = true;
        } else if pos.x >= bounds.max_x {
            pos.x = bounds.max_x;
            vel.x = -vel.x.abs();
            bounced = true;
        }

        if pos.y <= bounds.min_y {
            pos.y = bounds.min_y;
            vel.y = vel.y.abs();
            bounced = true;
        } else if pos.y >= bounds.max_y {
            pos.y = bounds.max_y;
            vel.y = -vel.y.abs();
            bounced = true;
        }

        let t = i as f32 * dt;
        keys.push(Keyframe::new(t, pos, Easing::Linear));
        if bounced {
            events.push(start + t);
        }
    }

    Ok((Track::new(keys)?, events, pos, vel))
}

fn temp_output_path(output_path: &Path, name: &str) -> PathBuf {
    let stem = output_path
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "render".to_string());
    match output_path.extension() {
        Some(ext) => output_path.with_file_name(format!("{stem}.{name}.{}", ext.to_string_lossy())),
        None => output_path.with_file_name(format!("{stem}.{name}")),
    }
}
