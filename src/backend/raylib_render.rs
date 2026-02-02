use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use raylib::consts::PixelFormat;
use raylib::prelude::*;

use crate::scene::{Color, Object, Shape, Transform, Vec2};
use crate::timeline::{SampledScene, Timeline};

pub struct RaylibRender {
    rl: RaylibHandle,
    thread: RaylibThread,
    render_texture: RenderTexture2D,
    width: u32,
    height: u32,
    bg: Color,
    cache: TextureCache,
}

impl RaylibRender {
    pub fn new(width: u32, height: u32, bg: Color) -> Result<Self> {
        let (mut rl, thread) = raylib::init()
            .size(width as i32, height as i32)
            .title("Rust Render (offline)")
            .build();

        let render_texture = rl
            .load_render_texture(&thread, width, height)
            .context("failed to create render texture")?;

        Ok(Self {
            rl,
            thread,
            render_texture,
            width,
            height,
            bg,
            cache: TextureCache::new(),
        })
    }

    pub fn render_timeline_rgba(
        &mut self,
        timeline: &Timeline,
        start_time: f32,
        end_time: f32,
        mut on_frame: impl FnMut(f32, &[u8]) -> Result<()>,
    ) -> Result<()> {
        if start_time < 0.0 || end_time <= start_time || end_time > timeline.duration {
            bail!("start/end time must satisfy 0 <= start < end <= duration");
        }

        let frames = ((end_time - start_time) * timeline.fps as f32).floor() as u32;
        for i in 0..frames {
            let t = start_time + i as f32 / timeline.fps as f32;
            let scene = timeline.sample(t)?;
            let rgba = self.render_scene_to_rgba(&scene)?;
            on_frame(t, &rgba)?;
        }

        Ok(())
    }

    pub fn render_scene_to_rgba(&mut self, scene: &SampledScene) -> Result<Vec<u8>> {
        self.cache.preload_for_scene(&mut self.rl, &self.thread, scene)?;

        {
            let mut d = self
                .rl
                .begin_texture_mode(&self.thread, self.render_texture.as_mut());
            d.clear_background(to_raylib_color(self.bg, 1.0));

            for layer in &scene.layers {
                for clip in &layer.clips {
                    draw_object(
                        &mut d,
                        &self.cache,
                        self.width,
                        self.height,
                        &clip.object,
                        &clip.transform,
                    )?;
                }
            }
        }

        capture_rgba(&self.render_texture, self.width, self.height)
    }
}

fn draw_object(
    d: &mut impl RaylibDraw,
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
    d: &mut impl RaylibDraw,
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
    d: &mut impl RaylibDraw,
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

fn capture_rgba(render_texture: &RenderTexture2D, expected_w: u32, expected_h: u32) -> Result<Vec<u8>> {
    let mut image = unsafe { raylib::ffi::LoadImageFromTexture(*render_texture.texture().as_ref()) };

    let result = (|| {
        if image.data.is_null() {
            bail!("raylib returned null image data");
        }

        if image.format != PixelFormat::PIXELFORMAT_UNCOMPRESSED_R8G8B8A8 as i32 {
            unsafe {
                raylib::ffi::ImageFormat(
                    &mut image,
                    PixelFormat::PIXELFORMAT_UNCOMPRESSED_R8G8B8A8 as i32,
                );
            }
        }

        if image.data.is_null() {
            bail!("image data was null after format conversion");
        }

        let width = image.width as u32;
        let height = image.height as u32;
        if width != expected_w || height != expected_h {
            bail!(
                "capture size mismatch: got {}x{}, expected {}x{}",
                width,
                height,
                expected_w,
                expected_h
            );
        }

        let len = (width * height * 4) as usize;
        let bytes = unsafe { std::slice::from_raw_parts(image.data as *const u8, len) };
        Ok(bytes.to_vec())
    })();

    unsafe {
        raylib::ffi::UnloadImage(image);
    }

    result
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
