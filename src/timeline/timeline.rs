use anyhow::{bail, Result};

use crate::timeline::{Clip, Layer};

#[derive(Debug, Clone, PartialEq)]
pub struct Timeline {
    pub duration: f32,
    pub fps: u32,
    pub layers: Vec<Layer>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SampledClip {
    pub object: crate::scene::Object,
    pub transform: crate::scene::Transform,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SampledLayer {
    pub name: String,
    pub clips: Vec<SampledClip>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SampledScene {
    pub layers: Vec<SampledLayer>,
}

impl Timeline {
    pub fn new(duration: f32, fps: u32) -> Result<Self> {
        if duration <= 0.0 {
            bail!("duration must be > 0");
        }
        if fps == 0 {
            bail!("fps must be > 0");
        }
        Ok(Self {
            duration,
            fps,
            layers: Vec::new(),
        })
    }

    pub fn add_layer(&mut self, layer: Layer) {
        self.layers.push(layer);
    }

    pub fn sample(&self, t: f32) -> Result<SampledScene> {
        if t < 0.0 || t > self.duration {
            bail!("sample time must be within 0..=duration");
        }

        let mut ordered: Vec<(usize, &Layer)> = self.layers.iter().enumerate().collect();
        ordered.sort_by(|(a_idx, a_layer), (b_idx, b_layer)| {
            let a_z = a_layer.z_override.unwrap_or(*a_idx as i32);
            let b_z = b_layer.z_override.unwrap_or(*b_idx as i32);
            a_z.cmp(&b_z).then_with(|| a_idx.cmp(b_idx))
        });

        let mut sampled_layers = Vec::with_capacity(ordered.len());
        for (_, layer) in ordered {
            let mut clips = Vec::new();
            for clip in &layer.clips {
                if clip.is_active(t) {
                    clips.push(SampledClip {
                        object: clip.object.clone(),
                        transform: clip.transform,
                    });
                }
            }
            sampled_layers.push(SampledLayer {
                name: layer.name.clone(),
                clips,
            });
        }

        Ok(SampledScene {
            layers: sampled_layers,
        })
    }

    pub fn total_frames(&self) -> u32 {
        (self.duration * self.fps as f32).floor() as u32
    }
}

impl Clip {
    pub fn validate_against(&self, duration: f32) -> Result<()> {
        if duration <= 0.0 {
            bail!("duration must be > 0");
        }
        if self.start < 0.0 || self.end <= self.start || self.end > duration {
            bail!("clip bounds must satisfy 0 <= start < end <= duration");
        }
        Ok(())
    }
}
