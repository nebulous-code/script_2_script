use std::env;
use std::path::{Path, PathBuf};

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
    let duration = 60.0;

    // Game of Life starts with a 2-second hold, then steps every half second.
    let mut timeline = Timeline::new(duration, fps)?;
    let args = RenderArgs::from_env(timeline.duration)?;

    // Grid sizing: 45x80 cells maps cleanly to a 540x960 canvas (12px per cell).
    let cols = 45;
    let rows = 80;
    let cell_size = width as f32 / cols as f32;
    // Leave a 1px gap so the grid reads clearly as squares.
    let cell_draw_size = cell_size - 1.0;

    // Create the initial grid and stamp "HELLO / WORLD!" into it.
    let mut grid = vec![false; cols * rows];
    seed_hello_world(&mut grid, cols, rows);

    // We hold the intro text for 2 seconds, then advance the simulation every 0.5s.
    let intro_hold = 2.0;
    let step_interval = 0.5;
    let segments = ((duration - intro_hold) / step_interval).floor() as usize;

    // Precompute all grid states so we can build timeline clips without re-simulating.
    let mut states = Vec::with_capacity(segments);
    let mut current = grid.clone();
    for _ in 0..segments {
        states.push(current.clone());
        current = step_grid(&current, cols, rows);
    }

    // Background layer fills the frame with a dark tone.
    let mut background = Layer::new("background");
    background.add_clip(Clip::new(
        0.0,
        duration,
        Object::Shape(Shape::Rect {
            width: width as f32,
            height: height as f32,
            color: Color::rgb(12, 12, 16),
        }),
        AnimatedTransform::constant(Transform::default()),
        duration,
    )?);

    // Game of Life cells are rendered as white squares.
    let mut life = Layer::new("life");
    let cell_color = Color::rgb(245, 245, 245);
    // First, draw the intro frame (static) for 2 seconds.
    for row in 0..rows {
        for col in 0..cols {
            if !grid[row * cols + col] {
                continue;
            }
            let pos = grid_to_world(col, row, cols, rows, cell_size);
            life.add_clip(Clip::new(
                0.0,
                intro_hold,
                Object::Shape(Shape::Rect {
                    width: cell_draw_size,
                    height: cell_draw_size,
                    color: cell_color,
                }),
                AnimatedTransform::constant(Transform {
                    pos,
                    ..Transform::default()
                }),
                duration,
            )?);
        }
    }

    // Then draw each simulation step as a short clip.
    for segment in 0..segments {
        let state = &states[segment];
        let start = intro_hold + segment as f32 * step_interval;
        let end = start + step_interval;

        for row in 0..rows {
            for col in 0..cols {
                if !state[row * cols + col] {
                    continue;
                }
                let pos = grid_to_world(col, row, cols, rows, cell_size);
                life.add_clip(Clip::new(
                    start,
                    end,
                    Object::Shape(Shape::Rect {
                        width: cell_draw_size,
                        height: cell_draw_size,
                        color: cell_color,
                    }),
                    AnimatedTransform::constant(Transform {
                        pos,
                        ..Transform::default()
                    }),
                    duration,
                )?);
            }
        }
    }

    timeline.add_layer(background);
    timeline.add_layer(life);

    // Preview uses a live window; render uses the ffmpeg pipeline.
    let preview = RaylibPreview::new(width, height, Color::rgb(12, 12, 16));

    if args.render {
        let output_path = args.resolve_output("game_of_life_demo")?;
        std::fs::create_dir_all(output_path.parent().unwrap_or(Path::new(".")))?;
        let mut renderer = RaylibRender::new(width, height, Color::rgb(12, 12, 16))?;
        let mut encoder = FfmpegVideoEncoder::start(width, height, fps, &output_path)?;
        renderer.render_timeline_rgba(&timeline, args.start_time, args.end_time, |_t, rgba| {
            encoder.write_frame(rgba)
        })?;
        encoder.finish()
    } else {
        preview.run_with(&timeline, args.start_time, args.end_time, |_| Ok(()))
    }
}

fn seed_hello_world(grid: &mut [bool], cols: usize, rows: usize) {
    // Two-line block text built from a tiny 3x5 pixel font.
    let lines = ["HELLO", "WORLD!"];
    let letter_width = 3;
    let letter_height = 5;
    let letter_spacing = 1;
    let space_width = 2;
    let line_spacing = 1;

    // Center the text block on the grid.
    let total_height = lines.len() * letter_height + (lines.len().saturating_sub(1) * line_spacing);
    let start_row = rows.saturating_sub(total_height) / 2;

    for (line_idx, line) in lines.iter().enumerate() {
        let line_width = measure_text_width(line, letter_width, letter_spacing, space_width);
        let start_col = cols.saturating_sub(line_width) / 2;
        let line_row = start_row + line_idx * (letter_height + line_spacing);

        let mut cursor = start_col;
        for ch in line.chars() {
            if ch == ' ' {
                cursor += space_width;
                continue;
            }

            if let Some(pattern) = letter_pattern(ch) {
                draw_letter(
                    grid,
                    cols,
                    rows,
                    cursor,
                    line_row,
                    letter_width,
                    letter_height,
                    pattern,
                );
            }

            cursor += letter_width + letter_spacing;
        }
    }
}

fn measure_text_width(
    text: &str,
    letter_width: usize,
    letter_spacing: usize,
    space_width: usize,
) -> usize {
    let mut width = 0;
    let mut first = true;
    for ch in text.chars() {
        if !first {
            width += letter_spacing;
        }
        first = false;
        if ch == ' ' {
            width += space_width;
        } else {
            width += letter_width;
        }
    }
    width
}

fn letter_pattern(ch: char) -> Option<[&'static str; 5]> {
    match ch {
        'H' => Some(["101", "101", "111", "101", "101"]),
        'E' => Some(["111", "100", "110", "100", "111"]),
        'L' => Some(["100", "100", "100", "100", "111"]),
        'O' => Some(["111", "101", "101", "101", "111"]),
        'W' => Some(["101", "101", "101", "111", "101"]),
        'R' => Some(["110", "101", "110", "101", "101"]),
        'D' => Some(["110", "101", "101", "101", "110"]),
        '!' => Some(["010", "010", "010", "000", "010"]),
        _ => None,
    }
}

fn draw_letter(
    grid: &mut [bool],
    cols: usize,
    rows: usize,
    start_col: usize,
    start_row: usize,
    letter_width: usize,
    letter_height: usize,
    pattern: [&str; 5],
) {
    // Each string row in `pattern` maps to pixels in the grid.
    for (row_offset, row) in pattern.iter().enumerate().take(letter_height) {
        for (col_offset, ch) in row.chars().enumerate().take(letter_width) {
            if ch != '1' {
                continue;
            }
            let col = start_col + col_offset;
            let row = start_row + row_offset;
            if col < cols && row < rows {
                grid[row * cols + col] = true;
            }
        }
    }
}

fn step_grid(current: &[bool], cols: usize, rows: usize) -> Vec<bool> {
    let mut next = vec![false; cols * rows];
    for row in 0..rows {
        for col in 0..cols {
            let mut neighbors = 0;
            for dy in [-1isize, 0, 1] {
                for dx in [-1isize, 0, 1] {
                    if dx == 0 && dy == 0 {
                        continue;
                    }
                    let nr = row as isize + dy;
                    let nc = col as isize + dx;
                    if nr < 0 || nc < 0 || nr >= rows as isize || nc >= cols as isize {
                        continue;
                    }
                    let idx = nr as usize * cols + nc as usize;
                    if current[idx] {
                        neighbors += 1;
                    }
                }
            }

            let idx = row * cols + col;
            let alive = current[idx];
            // Conway's rules: survive with 2-3 neighbors, born with exactly 3.
            next[idx] = if alive {
                neighbors == 2 || neighbors == 3
            } else {
                neighbors == 3
            };
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
