use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use raylib::prelude::*;

use crate::scene::{FontFamily, FontSource, Object, StyleFlags};
use crate::timeline::SampledScene;

pub struct ResourceCache {
    textures: HashMap<PathBuf, Texture2D>,
    fonts: HashMap<PathBuf, Font>,
    default_font: Option<WeakFont>,
}

impl ResourceCache {
    pub fn new() -> Self {
        Self {
            textures: HashMap::new(),
            fonts: HashMap::new(),
            default_font: None,
        }
    }

    pub fn set_default_font(&mut self, rl: &RaylibHandle) {
        self.default_font = Some(rl.get_font_default());
    }

    pub fn get_texture(&self, path: &Path) -> Result<&Texture2D> {
        if !path.exists() {
            bail!("image asset not found: {}", path.display());
        }
        Ok(self.textures.get(path).expect("texture cache missing"))
    }

    pub fn preload_for_scene(
        &mut self,
        rl: &mut RaylibHandle,
        thread: &RaylibThread,
        scene: &SampledScene,
    ) -> Result<()> {
        self.set_default_font(rl);
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
                if let Object::Text(text) = &clip.object {
                    self.preload_font_family(rl, thread, &text.font)?;
                }
            }
        }
        Ok(())
    }

    pub fn preload_font_family(
        &mut self,
        rl: &mut RaylibHandle,
        thread: &RaylibThread,
        family: &FontFamily,
    ) -> Result<()> {
        for source in [
            Some(&family.regular),
            family.bold.as_ref(),
            family.italic.as_ref(),
            family.bold_italic.as_ref(),
        ]
        .into_iter()
        .flatten()
        {
            if let FontSource::Path(path) = source {
                if !self.fonts.contains_key(path) {
                    let font = rl
                        .load_font(thread, path.to_string_lossy().as_ref())
                        .context("failed to load font")?;
                    self.fonts.insert(path.clone(), font);
                }
            }
        }
        Ok(())
    }

    pub fn resolve_font(&self, family: &FontFamily, style: StyleFlags) -> Result<FontRef<'_>> {
        match family.resolve(style) {
            FontSource::Default => {
                let font = self
                    .default_font
                    .as_ref()
                    .context("default font not set")?;
                Ok(FontRef::Default(font))
            }
            FontSource::Path(path) => {
                let font = self.fonts.get(path).context("font not loaded")?;
                Ok(FontRef::Loaded(font))
            }
        }
    }
}

#[derive(Clone, Copy)]
pub enum FontRef<'a> {
    Default(&'a WeakFont),
    Loaded(&'a Font),
}

pub fn measure_text(font: FontRef<'_>, text: &str, font_size: f32, spacing: f32) -> f32 {
    match font {
        FontRef::Default(font) => font.measure_text(text, font_size, spacing).x,
        FontRef::Loaded(font) => font.measure_text(text, font_size, spacing).x,
    }
}
