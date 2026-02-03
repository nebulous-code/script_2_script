use std::env;
use std::path::{Path, PathBuf};

use anyhow::{bail, Result};

use script_2_script::{
    AnimatedTransform, Clip, Color, FontFamily, FontSource, FfmpegVideoEncoder, Layer, Object,
    RaylibPreview, RaylibRender, StyledText, TextObject, Timeline, Track, Vec2,
};

fn main() -> Result<()> {
    // A simple credits roll that scrolls upward over 12 seconds.
    let mut timeline = Timeline::new(12.0, 30)?;
    let args = RenderArgs::from_env(timeline.duration)?;

    // Load markdown credits from assets/credits.md.
    let credits = std::fs::read_to_string("assets/credits.md")?;
    let styled = StyledText::from_markdown(&credits);

    // Text block setup: max_width controls wrapping width in pixels.
    let text_block = TextObject {
        text: styled,
        font: FontFamily {
            regular: FontSource::Path(PathBuf::from(
                "assets/Bodoni_Moda/BodoniModa_28pt-Regular.ttf",
            )),
            bold: Some(FontSource::Path(PathBuf::from(
                "assets/Bodoni_Moda/BodoniModa_28pt-Bold.ttf",
            ))),
            italic: Some(FontSource::Path(PathBuf::from(
                "assets/Bodoni_Moda/BodoniModa_28pt-Italic.ttf",
            ))),
            bold_italic: Some(FontSource::Path(PathBuf::from(
                "assets/Bodoni_Moda/BodoniModa_28pt-BoldItalic.ttf",
            ))),
        },
        font_size: 28.0,
        spacing: 1.0,
        max_width: 560.0,
        color: Color::rgb(230, 230, 230),
        line_spacing: 6.0,
    };

    let start_y = -300.0;
    let end_y = 320.0;
    let position_track = Track::new(vec![
        script_2_script::Keyframe::new(0.0, Vec2 { x: -280.0, y: start_y }, script_2_script::Easing::Linear),
        script_2_script::Keyframe::new(12.0, Vec2 { x: -280.0, y: end_y }, script_2_script::Easing::Linear),
    ])?;

    let text_transform = AnimatedTransform {
        position: position_track,
        scale: Track::from_constant(Vec2 { x: 1.0, y: 1.0 }),
        rotation: Track::from_constant(0.0),
        opacity: Track::from_constant(1.0),
    };

    let mut layer = Layer::new("credits");
    layer.add_clip(Clip::new(
        0.0,
        timeline.duration,
        Object::Text(text_block),
        text_transform,
        timeline.duration,
    )?);

    timeline.add_layer(layer);

    let preview = RaylibPreview::new(800, 600, Color::rgb(16, 16, 20));

    if args.render {
        let output_path = args.resolve_output("m6_credits_roll")?;
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
