use crate::timeline::Clip;

#[derive(Debug, Clone, PartialEq)]
pub struct Layer {
    pub name: String,
    pub z_override: Option<i32>,
    pub clips: Vec<Clip>,
}

impl Layer {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            z_override: None,
            clips: Vec::new(),
        }
    }

    pub fn with_z_override(mut self, z: i32) -> Self {
        self.z_override = Some(z);
        self
    }

    pub fn add_clip(&mut self, clip: Clip) {
        self.clips.push(clip);
    }
}
