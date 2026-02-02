pub mod audio;
pub mod backend;
pub mod encoder;
pub mod scene;
pub mod timeline;

pub use audio::{AudioEngine, MusicTrack, SfxEvent};
pub use backend::raylib_preview::RaylibPreview;
pub use backend::raylib_render::RaylibRender;
pub use encoder::{mux_video_audio, render_audio_wav, trim_audio, FfmpegVideoEncoder};
pub use scene::{
    AnimatedTransform, Color, Easing, ImageObject, Keyframe, Object, Shape, Track, Transform, Vec2,
};
pub use timeline::{Clip, Layer, Timeline};
