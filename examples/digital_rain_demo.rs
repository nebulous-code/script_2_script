use std::env;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::Local;
use anyhow::{bail, Result};

use script_2_script::{
    AnimatedTransform, Clip, Color, FfmpegVideoEncoder, Layer, Object, RaylibPreview,
    RaylibRender, Shape, Timeline, Transform, Vec2,
};

fn main() -> Result<()> {
    // TikTok-friendly vertical canvas.
    let width = 540;
    let height = 960;
    let fps = 30;
    let duration = 20.0;

    // Digital rain "pixels" laid out on a grid.
    let cols = 45;
    let rows = 80;
    let cell_size = width as f32 / cols as f32;
    let cell_draw_size = cell_size - 1.0;

    let args = RenderArgs::from_env(duration)?;
    let duration = args.duration;
    let mut timeline = Timeline::new(duration, fps)?;

    let steps = (duration / args.step_interval).ceil() as usize;
    let mut grid = vec![Cell::default(); cols * rows];
    println!("seed: {}", args.seed);
    let mut rng = SimpleRng::new(args.seed);
    let mut spawn_accumulator = 0.0f32;

    // Background layer fills the frame with a dark tone.
    let mut background = Layer::new("background");
    background.add_clip(Clip::new(
        0.0,
        duration,
        Object::Shape(Shape::Rect {
            width: width as f32,
            height: height as f32,
            color: Color::rgb(10, 10, 14),
        }),
        AnimatedTransform::constant(Transform::default()),
        duration,
    )?);

    // Digital rain layer: colored squares that dim and fall.
    let mut rain = Layer::new("rain");

    for step in 0..steps {
        let start = step as f32 * args.step_interval;
        let end = (start + args.step_interval).min(duration);

        // Spawn new drops at random positions each step.
        let spawn_count = if let Some(rate) = args.spawn_rate {
            spawn_accumulator += rate.max(0.0) * args.step_interval;
            let count = spawn_accumulator.floor() as usize;
            spawn_accumulator -= count as f32;
            count
        } else {
            args.spawn_per_step
        };

        if spawn_count > 0 {
            spawn_drops(
                &mut grid,
                cols,
                rows,
                spawn_count,
                &mut rng,
                args.red_scale,
                args.green_scale,
                args.blue_scale,
                &args,
            );
        }

        // Draw the current grid state for this time slice.
        for row in 0..rows {
            for col in 0..cols {
                let cell = &grid[row * cols + col];
                if cell.brightness <= 0.02 {
                    continue;
                }

                let pos = grid_to_world(col, row, cols, rows, cell_size);
                let color = cell.to_color();
                rain.add_clip(Clip::new(
                    start,
                    end,
                    Object::Shape(Shape::Rect {
                        width: cell_draw_size,
                        height: cell_draw_size,
                        color,
                    }),
                    AnimatedTransform::constant(Transform {
                        pos,
                        ..Transform::default()
                    }),
                    duration,
                )?);
            }
        }

        // Advance the simulation for the next step.
        let dim_value = args.resolve_dim(&mut rng);
        grid = step_rain(&grid, cols, rows, dim_value);
    }

    timeline.add_layer(background);
    timeline.add_layer(rain);

    // Preview uses a live window; render uses the ffmpeg pipeline.
    let preview = RaylibPreview::new(width, height, Color::rgb(10, 10, 14));

    if args.render {
        let output_path = args.resolve_output("digital_rain_demo")?;
        std::fs::create_dir_all(output_path.parent().unwrap_or(Path::new(".")))?;
        let mut renderer = RaylibRender::new(width, height, Color::rgb(10, 10, 14))?;
        let mut encoder = FfmpegVideoEncoder::start(width, height, fps, &output_path)?;
        renderer.render_timeline_rgba(&timeline, args.start_time, args.end_time, |_t, rgba| {
            encoder.write_frame(rgba)
        })?;
        encoder.finish()
    } else {
        preview.run_with(&timeline, args.start_time, args.end_time, |_| Ok(()))
    }
}

#[derive(Clone, Copy)]
struct Cell {
    r: u8,
    g: u8,
    b: u8,
    brightness: f32,
    fall_rate: f32,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            r: 0,
            g: 0,
            b: 0,
            brightness: 0.0,
            fall_rate: 0.0,
        }
    }
}

impl Cell {
    fn to_color(&self) -> Color {
        let scale = self.brightness.clamp(0.0, 1.0);
        let r = (self.r as f32 * scale).round().clamp(0.0, 255.0) as u8;
        let g = (self.g as f32 * scale).round().clamp(0.0, 255.0) as u8;
        let b = (self.b as f32 * scale).round().clamp(0.0, 255.0) as u8;
        Color::rgb(r, g, b)
    }
}

fn spawn_drops(
    grid: &mut [Cell],
    cols: usize,
    rows: usize,
    count: usize,
    rng: &mut SimpleRng,
    red_scale: f32,
    green_scale: f32,
    blue_scale: f32,
    args: &RenderArgs,
) {
    for _ in 0..count {
        let col = rng.next_usize(cols);
        let row = rng.next_usize(rows);
        let idx = row * cols + col;
        let color = random_color(rng, red_scale, green_scale, blue_scale);
        let fall_rate = args.resolve_fall(rng);
        apply_cell(grid, idx, color, 1.0, fall_rate);
    }
}

fn random_color(rng: &mut SimpleRng, red_scale: f32, green_scale: f32, blue_scale: f32) -> (u8, u8, u8) {
    let r = scale_channel(rng.next_u8(), red_scale);
    let g = scale_channel(rng.next_u8(), green_scale);
    let b = scale_channel(rng.next_u8(), blue_scale);
    (r, g, b)
}

fn scale_channel(value: u8, scale: f32) -> u8 {
    let scaled = value as f32 * scale.clamp(0.0, 1.0);
    scaled.round().clamp(0.0, 255.0) as u8
}

fn apply_cell(grid: &mut [Cell], idx: usize, color: (u8, u8, u8), brightness: f32, fall_rate: f32) {
    let cell = &mut grid[idx];
    if brightness >= cell.brightness {
        cell.r = color.0;
        cell.g = color.1;
        cell.b = color.2;
        cell.brightness = brightness;
        cell.fall_rate = fall_rate;
    } else {
        cell.brightness = cell.brightness.max(brightness);
    }
}

fn step_rain(current: &[Cell], cols: usize, rows: usize, dim: f32) -> Vec<Cell> {
    let mut next = vec![Cell::default(); cols * rows];
    let dim = dim.clamp(0.0, 1.0);

    for row in 0..rows {
        for col in 0..cols {
            let idx = row * cols + col;
            let cell = current[idx];
            if cell.brightness <= 0.0 {
                continue;
            }

            // Fall to the cell below (each drop carries its own fall rate).
            if row + 1 < rows {
                let below = (row + 1) * cols + col;
                let fall = cell.fall_rate.clamp(0.0, 1.0);
                let fallen = cell.brightness * fall;
                apply_cell(
                    &mut next,
                    below,
                    (cell.r, cell.g, cell.b),
                    fallen,
                    cell.fall_rate,
                );
            }

            // Dim the current cell to leave a trail that always fades faster than the drop.
            let mut dimmed = cell.brightness * dim;
            let fallen = cell.brightness * cell.fall_rate.clamp(0.0, 1.0);
            if dimmed >= fallen {
                dimmed = fallen * 0.8;
            }
            apply_cell(
                &mut next,
                idx,
                (cell.r, cell.g, cell.b),
                dimmed,
                cell.fall_rate,
            );
        }
    }

    next
}

fn grid_to_world(col: usize, row: usize, cols: usize, rows: usize, cell_size: f32) -> Vec2 {
    // Convert grid coordinates to scene coordinates (origin at the center).
    // We flip the Y axis so row 0 maps to the top of the screen.
    let row = rows.saturating_sub(1).saturating_sub(row);
    let total_w = cols as f32 * cell_size;
    let total_h = rows as f32 * cell_size;
    let x = -total_w / 2.0 + cell_size / 2.0 + col as f32 * cell_size;
    let y = -total_h / 2.0 + cell_size / 2.0 + row as f32 * cell_size;
    Vec2 { x, y }
}

struct SimpleRng {
    state: u64,
}

impl SimpleRng {
    fn new(seed: u64) -> Self {
        let seed = if seed == 0 { 0x1234_5678_9ABC_DEF0 } else { seed };
        Self { state: seed }
    }

    fn next_u32(&mut self) -> u32 {
        // LCG parameters from Numerical Recipes.
        self.state = self.state.wrapping_mul(1664525).wrapping_add(1013904223);
        (self.state >> 16) as u32
    }

    fn next_u8(&mut self) -> u8 {
        (self.next_u32() & 0xFF) as u8
    }

    fn next_usize(&mut self, max: usize) -> usize {
        if max == 0 {
            return 0;
        }
        (self.next_u32() as usize) % max
    }

    fn next_f32(&mut self) -> f32 {
        let value = self.next_u32();
        value as f32 / u32::MAX as f32
    }
}

struct RenderArgs {
    render: bool,
    start_time: f32,
    end_time: f32,
    output: Option<PathBuf>,
    duration: f32,
    dim_default: f32,
    dim_fixed: Option<f32>,
    dim_min: Option<f32>,
    dim_max: Option<f32>,
    fall_default: f32,
    fall_fixed: Option<f32>,
    fall_min: Option<f32>,
    fall_max: Option<f32>,
    step_interval: f32,
    spawn_per_step: usize,
    spawn_rate: Option<f32>,
    seed: u64,
    red_scale: f32,
    green_scale: f32,
    blue_scale: f32,
}

impl RenderArgs {
    fn from_env(default_duration: f32) -> Result<Self> {
        let mut render = false;
        let mut start_time = 0.0;
        let mut duration = default_duration;
        let mut end_time = duration;
        let mut output = None;
        let mut dim_default = 0.7;
        let mut dim_fixed = None;
        let mut dim_min = None;
        let mut dim_max = None;
        let mut fall_default = 0.5;
        let mut fall_fixed = None;
        let mut fall_min = None;
        let mut fall_max = None;
        let mut step_interval = 0.1;
        let mut spawn_per_step = 35;
        let mut spawn_rate = None;
        let mut seed = default_seed();
        let mut red_scale = 1.0;
        let mut green_scale = 1.0;
        let mut blue_scale = 1.0;

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
                "--duration" => {
                    let value = args
                        .next()
                        .ok_or_else(|| anyhow::anyhow!("--duration requires a value"))?;
                    duration = value.parse::<f32>()?;
                    end_time = duration;
                }
                "--dim" => {
                    let value = args
                        .next()
                        .ok_or_else(|| anyhow::anyhow!("--dim requires a value"))?;
                    dim_fixed = Some(value.parse::<f32>()?);
                }
                "--dim_min" => {
                    let value = args
                        .next()
                        .ok_or_else(|| anyhow::anyhow!("--dim_min requires a value"))?;
                    dim_min = Some(value.parse::<f32>()?);
                }
                "--dim_max" => {
                    let value = args
                        .next()
                        .ok_or_else(|| anyhow::anyhow!("--dim_max requires a value"))?;
                    dim_max = Some(value.parse::<f32>()?);
                }
                "--fall" => {
                    let value = args
                        .next()
                        .ok_or_else(|| anyhow::anyhow!("--fall requires a value"))?;
                    fall_fixed = Some(value.parse::<f32>()?);
                }
                "--fall_min" => {
                    let value = args
                        .next()
                        .ok_or_else(|| anyhow::anyhow!("--fall_min requires a value"))?;
                    fall_min = Some(value.parse::<f32>()?);
                }
                "--fall_max" => {
                    let value = args
                        .next()
                        .ok_or_else(|| anyhow::anyhow!("--fall_max requires a value"))?;
                    fall_max = Some(value.parse::<f32>()?);
                }
                "--step" => {
                    let value = args
                        .next()
                        .ok_or_else(|| anyhow::anyhow!("--step requires a value"))?;
                    step_interval = value.parse::<f32>()?;
                }
                "--spawn" => {
                    let value = args
                        .next()
                        .ok_or_else(|| anyhow::anyhow!("--spawn requires a value"))?;
                    spawn_per_step = value.parse::<usize>()?;
                }
                "--spawn_rate" => {
                    let value = args
                        .next()
                        .ok_or_else(|| anyhow::anyhow!("--spawn_rate requires a value"))?;
                    spawn_rate = Some(value.parse::<f32>()?);
                }
                "--seed" => {
                    let value = args
                        .next()
                        .ok_or_else(|| anyhow::anyhow!("--seed requires a value"))?;
                    seed = value.parse::<u64>()?;
                }
                "--r" => {
                    let value = args
                        .next()
                        .ok_or_else(|| anyhow::anyhow!("--r requires a value"))?;
                    red_scale = value.parse::<f32>()?;
                }
                "--g" => {
                    let value = args
                        .next()
                        .ok_or_else(|| anyhow::anyhow!("--g requires a value"))?;
                    green_scale = value.parse::<f32>()?;
                }
                "--b" => {
                    let value = args
                        .next()
                        .ok_or_else(|| anyhow::anyhow!("--b requires a value"))?;
                    blue_scale = value.parse::<f32>()?;
                }
                "--preview" => {}
                other => bail!("unknown argument: {other}"),
            }
        }

        if duration <= 0.0 {
            bail!("--duration must be > 0");
        }

        if start_time < 0.0 || end_time <= start_time || end_time > duration {
            bail!("start/end time must satisfy 0 <= start < end <= duration");
        }

        if step_interval <= 0.0 {
            bail!("--step must be > 0");
        }

        Ok(Self {
            render,
            start_time,
            end_time,
            output,
            duration,
            dim_default,
            dim_fixed,
            dim_min,
            dim_max,
            fall_default,
            fall_fixed,
            fall_min,
            fall_max,
            step_interval,
            spawn_per_step,
            spawn_rate,
            seed,
            red_scale,
            green_scale,
            blue_scale,
        })
    }

    fn resolve_output(&self, default_stem: &str) -> Result<PathBuf> {
        if let Some(path) = &self.output {
            return Ok(path.clone());
        }
        let stamp = Local::now().format("%Y%m%d%H%M%S");
        Ok(PathBuf::from(format!("output/{default_stem}_{stamp}.mp4")))
    }

    fn resolve_dim(&self, rng: &mut SimpleRng) -> f32 {
        if let Some(fixed) = self.dim_fixed {
            return fixed.clamp(0.0, 1.0);
        }

        if self.dim_min.is_some() || self.dim_max.is_some() {
            let mut min = self.dim_min.unwrap_or(self.dim_default);
            let mut max = self.dim_max.unwrap_or(self.dim_default);
            if min > max {
                std::mem::swap(&mut min, &mut max);
            }
            let t = rng.next_f32();
            return (min + (max - min) * t).clamp(0.0, 1.0);
        }

        self.dim_default.clamp(0.0, 1.0)
    }

    fn resolve_fall(&self, rng: &mut SimpleRng) -> f32 {
        if let Some(fixed) = self.fall_fixed {
            return fixed.clamp(0.0, 1.0);
        }

        if self.fall_min.is_some() || self.fall_max.is_some() {
            let mut min = self.fall_min.unwrap_or(self.fall_default);
            let mut max = self.fall_max.unwrap_or(self.fall_default);
            if min > max {
                std::mem::swap(&mut min, &mut max);
            }
            let t = rng.next_f32();
            return (min + (max - min) * t).clamp(0.0, 1.0);
        }

        self.fall_default.clamp(0.0, 1.0)
    }
}

fn default_seed() -> u64 {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    (nanos & u64::MAX as u128) as u64
}
