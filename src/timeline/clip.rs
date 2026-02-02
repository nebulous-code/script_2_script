use anyhow::{bail, Result};

use crate::scene::{Object, Transform};

#[derive(Debug, Clone, PartialEq)]
pub struct Clip {
    pub start: f32,
    pub end: f32,
    pub object: Object,
    pub transform: Transform,
}

impl Clip {
    pub fn new(start: f32, end: f32, object: Object, transform: Transform, duration: f32) -> Result<Self> {
        if duration <= 0.0 {
            bail!("duration must be > 0");
        }
        if start < 0.0 || end <= start || end > duration {
            bail!("clip bounds must satisfy 0 <= start < end <= duration");
        }
        Ok(Self {
            start,
            end,
            object,
            transform,
        })
    }

    pub fn is_active(&self, t: f32) -> bool {
        t >= self.start && t < self.end
    }
}
