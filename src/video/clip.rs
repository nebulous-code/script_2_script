use std::path::PathBuf;

use anyhow::{bail, Result};

#[derive(Debug, Clone, PartialEq)]
pub struct VideoClip {
    pub path: PathBuf,
    pub start_time: f32,
    pub end_time: f32,
    pub trim_start: Option<f32>,
    pub trim_end: Option<f32>,
}

impl VideoClip {
    pub fn new(
        path: impl Into<PathBuf>,
        start_time: f32,
        end_time: f32,
        trim_start: Option<f32>,
        trim_end: Option<f32>,
    ) -> Result<Self> {
        let path = path.into();
        if !path.exists() {
            bail!("video clip not found: {}", path.display());
        }
        if start_time < 0.0 || end_time <= start_time {
            bail!("clip bounds must satisfy 0 <= start < end");
        }
        if let Some(ts) = trim_start {
            if ts < 0.0 {
                bail!("trim_start must be >= 0");
            }
        }
        if let Some(te) = trim_end {
            if te <= 0.0 {
                bail!("trim_end must be > 0");
            }
        }
        if let (Some(ts), Some(te)) = (trim_start, trim_end) {
            if te <= ts {
                bail!("trim_end must be > trim_start");
            }
            let clip_len = end_time - start_time;
            if te - ts < clip_len {
                bail!("trim range shorter than clip duration");
            }
        }

        Ok(Self {
            path,
            start_time,
            end_time,
            trim_start,
            trim_end,
        })
    }

    pub fn duration(&self) -> f32 {
        self.end_time - self.start_time
    }
}
