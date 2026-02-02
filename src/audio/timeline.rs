use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq)]
pub struct MusicTrack {
    pub path: PathBuf,
    pub start: f32,
    pub end: f32,
    pub looped: bool,
    pub volume: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SfxEvent {
    pub path: PathBuf,
    pub time: f32,
    pub volume: f32,
}
