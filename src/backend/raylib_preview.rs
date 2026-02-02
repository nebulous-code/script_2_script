use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use raylib::prelude::*;

use crate::scene::{Color, Object, Shape, Transform, Vec2};
use crate::timeline::{SampledScene, Timeline};

pub struct RaylibPreview {
    width: u32,
    height: u32,
    bg: Color,
}

impl RaylibPreview {
    pub fn new(width: u32, height: u32, bg: Color) -> Self {
        Self { width, height, bg }
    }

    pub fn run(&self, timeline: &Timeline) -> Result<()> {
        let (mut rl, thread) = raylib::init()
            .size(self.width as i32, self.height as i32)
            .title("Rust Render Preview")
            .build();

        rl.set_target_fps(timeline.fps);
        let mut cache = TextureCache::new();
        let total_frames = timeline.total_frames();

        for frame_index in 0..total_frames {
            if rl.window_should_close() {
                break;
            }
            let t = frame_index as f32 / timeline.fps as f32;
            let scene = timeline.sample(t)?;
            self.draw_scene(&mut rl, &thread, &mut cache, &scene)?;
        }

        Ok(())
    }

    fn draw_scene(
        &self,
        rl: &mut RaylibHandle,
        thread: &RaylibThread,
        cache: &mut TextureCache,
        scene: &SampledScene,
    ) -> Result<()> {
        cache.preload_for_scene(rl, thread, scene)?;

        let mut d = rl.begin_drawing(thread);
        d.clear_background(to_raylib_color(self.bg, 1.0));

        for layer in &scene.layers {
            for clip in &layer.clips {
                draw_object(
                    &mut d,
                    cache,
                    self.width,
                    self.height,
                    &clip.object,
                    &clip.transform,
                )?;
            }
        }

        Ok(())
    }
}

fn draw_object(
    d: &mut RaylibDrawHandle,
    cache: &TextureCache,
    width: u32,
    height: u32,
    object: &Object,
    transform: &Transform,
) -> Result<()> {
    match object {
        Object::Shape(shape) => draw_shape(d, width, height, shape, transform),
        Object::Image(image) => draw_image(d, cache, width, height, &image.path, transform),
    }
}

fn draw_shape(
    d: &mut RaylibDrawHandle,
    width: u32,
    height: u32,
    shape: &Shape,
    transform: &Transform,
) -> Result<()> {
    let center = graph_to_screen(transform.pos, width, height);
    let color = to_raylib_color(
        match shape {
            Shape::Circle { color, .. } => *color,
            Shape::Rect { color, .. } => *color,
        },
        transform.opacity,
    );

    match shape {
        Shape::Circle { radius, .. } => {
            let scaled = radius * transform.scale.x.max(0.0);
            d.draw_circle_v(center, scaled, color);
        }
        Shape::Rect { width: w, height: h, .. } => {
            let w = w * transform.scale.x;
            let h = h * transform.scale.y;
            let rec = Rectangle::new(center.x, center.y, w, h);
            let origin = Vector2::new(w / 2.0, h / 2.0);
            d.draw_rectangle_pro(rec, origin, transform.rotation, color);
        }
    }

    Ok(())
}

fn draw_image(
    d: &mut RaylibDrawHandle,
    cache: &TextureCache,
    width: u32,
    height: u32,
    path: &Path,
    transform: &Transform,
) -> Result<()> {
    let texture = cache.get(path)?;
    let tex_w = texture.width as f32;
    let tex_h = texture.height as f32;

    let w = tex_w * transform.scale.x;
    let h = tex_h * transform.scale.y;
    let center = graph_to_screen(transform.pos, width, height);

    let source = Rectangle::new(0.0, 0.0, tex_w, tex_h);
    let dest = Rectangle::new(center.x, center.y, w, h);
    let origin = Vector2::new(w / 2.0, h / 2.0);

    let tint = to_raylib_color(Color::WHITE, transform.opacity);
    d.draw_texture_pro(texture, source, dest, origin, transform.rotation, tint);
    Ok(())
}

fn graph_to_screen(pos: Vec2, width: u32, height: u32) -> Vector2 {
    Vector2::new(width as f32 / 2.0 + pos.x, height as f32 / 2.0 - pos.y)
}

fn to_raylib_color(color: Color, opacity: f32) -> raylib::prelude::Color {
    let alpha = (color.a as f32 * opacity.clamp(0.0, 1.0))
        .round()
        .clamp(0.0, 255.0) as u8;
    raylib::prelude::Color::new(color.r, color.g, color.b, alpha)
}

struct TextureCache {
    textures: HashMap<PathBuf, Texture2D>,
}

impl TextureCache {
    fn new() -> Self {
        Self {
            textures: HashMap::new(),
        }
    }

    fn get(&self, path: &Path) -> Result<&Texture2D> {
        if !path.exists() {
            bail!("image asset not found: {}", path.display());
        }
        Ok(self.textures.get(path).expect("texture cache missing"))
    }

    fn preload_for_scene(
        &mut self,
        rl: &mut RaylibHandle,
        thread: &RaylibThread,
        scene: &SampledScene,
    ) -> Result<()> {
        for layer in &scene.layers {
            for clip in &layer.clips {
                if let Object::Image(image) = &clip.object {
                    let path = &image.path;
                    if !path.exists() {
                        bail!("image asset not found: {}", path.display());
                    }
                    if !self.textures.contains_key(path) {
                        let tex = rl
                            .load_texture(thread, path.to_string_lossy().as_ref())
                            .context("failed to load texture")?;
                        self.textures.insert(path.to_path_buf(), tex);
                    }
                }
            }
        }
        Ok(())
    }
}
