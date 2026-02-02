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
    let start_pos = Vec2 { x: -200.0, y: 80.0 };
    let start_vel = Vec2 { x: 220.0, y: 170.0 };

    // Cycle through red → yellow → green → cyan → blue → magenta.
    let colors = [
        Color::rgb(255, 0, 0),
        Color::rgb(255, 255, 0),
        Color::rgb(0, 255, 0),
        Color::rgb(0, 255, 255),
        Color::rgb(0, 0, 255),
        Color::rgb(255, 0, 255),
    ];

    // Precompute the full bounce path and SFX events for the whole timeline.
    let samples =
        build_bounce_samples(timeline.duration, timeline.fps, start_pos, start_vel, bounds);

    let sfx_events = samples
        .bounce_times
        .iter()
        .map(|time| SfxEvent {
            path: PathBuf::from("assets/border.ogg"),
            time: *time,
            volume: 0.7,
        })
        .collect::<Vec<_>>();

    // Split the timeline into color segments, with cross-fades.
    let segment = timeline.duration / colors.len() as f32;
    let fade = (segment * 0.2).min(1.0);

    for (i, color) in colors.iter().enumerate() {
        let base_start = i as f32 * segment;
        let base_end = if i == colors.len() - 1 {
            timeline.duration
        } else {
            (i + 1) as f32 * segment
        };

        let clip_start = (base_start - fade).max(0.0);
        let clip_end = (base_end + fade).min(timeline.duration);
        let fade_in = base_start - clip_start;
        let fade_out = clip_end - base_end;

        let position = track_from_samples(&samples.positions, clip_start, clip_end, timeline.fps)?;
        let opacity = opacity_track(clip_end - clip_start, fade_in, fade_out)?;

        motion.add_clip(Clip::new(
            clip_start,
            clip_end,
            Object::Shape(Shape::Circle {
                radius,
                color: *color,
            }),
            AnimatedTransform {
                position,
                scale: Track::from_constant(Vec2 { x: 1.0, y: 1.0 }),
                rotation: Track::from_constant(0.0),
                opacity,
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

struct BounceSamples {
    positions: Vec<Vec2>,
    bounce_times: Vec<f32>,
}

fn build_bounce_samples(
    duration: f32,
    fps: u32,
    mut pos: Vec2,
    mut vel: Vec2,
    bounds: Bounds,
) -> BounceSamples {
    let dt = 1.0 / fps as f32;
    let frames = (duration * fps as f32).floor() as u32;
    let mut positions = Vec::with_capacity(frames as usize + 1);
    let mut bounce_times = Vec::new();
    positions.push(pos);

    for i in 0..frames {
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

        let t = (i + 1) as f32 * dt;
        positions.push(pos);
        if bounced {
            bounce_times.push(t);
        }
    }

    BounceSamples {
        positions,
        bounce_times,
    }
}

fn track_from_samples(
    positions: &[Vec2],
    start: f32,
    end: f32,
    fps: u32,
) -> Result<Track<Vec2>> {
    let start_idx = (start * fps as f32).floor() as usize;
    let end_idx = (end * fps as f32).floor() as usize;
    let dt = 1.0 / fps as f32;
    let mut keys = Vec::with_capacity(end_idx.saturating_sub(start_idx) + 1);

    for i in start_idx..=end_idx.min(positions.len() - 1) {
        let t = (i - start_idx) as f32 * dt;
        keys.push(Keyframe::new(t, positions[i], Easing::Linear));
    }

    Track::new(keys)
}

fn opacity_track(duration: f32, fade_in: f32, fade_out: f32) -> Result<Track<f32>> {
    let mut keys = Vec::new();
    if fade_in > 0.0 {
        keys.push(Keyframe::new(0.0, 0.0, Easing::Linear));
        keys.push(Keyframe::new(fade_in, 1.0, Easing::Linear));
    } else {
        keys.push(Keyframe::new(0.0, 1.0, Easing::Linear));
    }

    if fade_out > 0.0 {
        let start = (duration - fade_out).max(0.0);
        if start > keys.last().map(|k| k.time).unwrap_or(0.0) {
            keys.push(Keyframe::new(start, 1.0, Easing::Linear));
        }
        keys.push(Keyframe::new(duration, 0.0, Easing::Linear));
    }

    Track::new(keys)
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
