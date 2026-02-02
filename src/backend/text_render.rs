use anyhow::Result;
use raylib::prelude::*;

use crate::backend::resources::{measure_text, FontRef, ResourceCache};
use crate::scene::{StyleFlags, TextObject, TextRun, Transform, Vec2};

pub struct LineLayout {
    pub runs: Vec<TextRun>,
}

pub fn draw_text_block(
    d: &mut impl RaylibDraw,
    cache: &ResourceCache,
    width: u32,
    height: u32,
    text: &TextObject,
    transform: &Transform,
) -> Result<()> {
    let origin = graph_to_screen(transform.pos, width, height);
    let font_size = text.font_size * transform.scale.y.max(0.0);
    let spacing = text.spacing;
    let line_height = font_size + text.line_spacing;

    let lines = layout_text(text, cache, font_size, spacing)?;

    let mut y = origin.y;
    for line in lines {
        let mut x = origin.x;
        for run in line.runs {
            let font = cache.resolve_font(&text.font, run.style)?;
            let tint = to_raylib_color(text.color, transform.opacity);
            let position = Vector2::new(x, y);
            let origin_vec = Vector2::new(0.0, 0.0);
            draw_text_pro(
                d,
                font,
                &run.text,
                position,
                origin_vec,
                transform.rotation,
                font_size,
                spacing,
                tint,
            );

            let width = measure_text(font, &run.text, font_size, spacing);
            if run.style.underline {
                let underline_y = y + font_size * 0.9;
                d.draw_line_ex(
                    Vector2::new(x, underline_y),
                    Vector2::new(x + width, underline_y),
                    2.0,
                    tint,
                );
            }

            x += width;
        }
        y += line_height;
    }

    Ok(())
}

pub fn layout_text(
    text: &TextObject,
    cache: &ResourceCache,
    font_size: f32,
    spacing: f32,
) -> Result<Vec<LineLayout>> {
    let max_width = if text.max_width <= 0.0 {
        f32::INFINITY
    } else {
        text.max_width
    };
    let mut lines = Vec::new();
    let mut current = LineLayout { runs: Vec::new() };
    let mut line_width = 0.0;

    for run in &text.text.runs {
        let parts = split_newlines(&run.text);
        for (idx, part) in parts.iter().enumerate() {
            if idx > 0 {
                lines.push(current);
                current = LineLayout { runs: Vec::new() };
                line_width = 0.0;
            }

            for token in split_tokens(part) {
                let token_width =
                    measure_token(cache, &text, run.style, &token, font_size, spacing)?;

                if line_width + token_width <= max_width || line_width == 0.0 {
                    push_run(&mut current.runs, run.style, &token);
                    line_width += token_width;
                    continue;
                }

                if token_width > max_width && line_width == 0.0 {
                    for ch in token.chars() {
                        let s = ch.to_string();
                        let w = measure_token(cache, &text, run.style, &s, font_size, spacing)?;
                        if line_width + w > max_width && line_width > 0.0 {
                            lines.push(current);
                            current = LineLayout { runs: Vec::new() };
                            line_width = 0.0;
                        }
                        push_run(&mut current.runs, run.style, &s);
                        line_width += w;
                    }
                    continue;
                }

                lines.push(current);
                current = LineLayout { runs: Vec::new() };
                line_width = 0.0;
                if token.trim().is_empty() {
                    continue;
                }
                push_run(&mut current.runs, run.style, &token);
                line_width += token_width;
            }
        }
    }

    lines.push(current);
    Ok(lines)
}

fn push_run(runs: &mut Vec<TextRun>, style: StyleFlags, text: &str) {
    if text.is_empty() {
        return;
    }
    if let Some(last) = runs.last_mut() {
        if last.style == style {
            last.text.push_str(text);
            return;
        }
    }
    runs.push(TextRun {
        text: text.to_string(),
        style,
    });
}

fn split_newlines(text: &str) -> Vec<String> {
    text.split('\n').map(|s| s.to_string()).collect()
}

fn split_tokens(text: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut buf = String::new();
    let mut last_space = None;
    for ch in text.chars() {
        let is_space = ch.is_whitespace();
        match last_space {
            None => {
                last_space = Some(is_space);
                buf.push(ch);
            }
            Some(was_space) if was_space == is_space => buf.push(ch),
            Some(_) => {
                tokens.push(buf.clone());
                buf.clear();
                buf.push(ch);
                last_space = Some(is_space);
            }
        }
    }
    if !buf.is_empty() {
        tokens.push(buf);
    }
    tokens
}

fn measure_token(
    cache: &ResourceCache,
    text: &TextObject,
    style: StyleFlags,
    token: &str,
    font_size: f32,
    spacing: f32,
) -> Result<f32> {
    let font = cache.resolve_font(&text.font, style)?;
    Ok(measure_text(font, token, font_size, spacing))
}

fn draw_text_pro(
    d: &mut impl RaylibDraw,
    font: FontRef<'_>,
    text: &str,
    position: Vector2,
    origin: Vector2,
    rotation: f32,
    font_size: f32,
    spacing: f32,
    tint: raylib::prelude::Color,
) {
    match font {
        FontRef::Default(font) => {
            d.draw_text_pro(font, text, position, origin, rotation, font_size, spacing, tint)
        }
        FontRef::Loaded(font) => d.draw_text_pro(font, text, position, origin, rotation, font_size, spacing, tint),
    }
}

fn graph_to_screen(pos: Vec2, width: u32, height: u32) -> Vector2 {
    Vector2::new(width as f32 / 2.0 + pos.x, height as f32 / 2.0 - pos.y)
}

fn to_raylib_color(color: crate::scene::Color, opacity: f32) -> raylib::prelude::Color {
    let alpha = (color.a as f32 * opacity.clamp(0.0, 1.0))
        .round()
        .clamp(0.0, 255.0) as u8;
    raylib::prelude::Color::new(color.r, color.g, color.b, alpha)
}
