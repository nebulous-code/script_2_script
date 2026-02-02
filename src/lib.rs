pub mod audio;
pub mod backend;
pub mod encoder;
pub mod scene;
pub mod timeline;
pub mod video;

pub use audio::{AudioEngine, MusicTrack, SfxEvent};
pub use backend::raylib_preview::RaylibPreview;
pub use backend::raylib_render::RaylibRender;
pub use encoder::{
    build_base_video, mux_video_audio, render_audio_wav, trim_audio, FfmpegVideoEncoder,
};
pub use video::{resolve_segments, VideoClip, VideoSegment};
pub use scene::{
    AnimatedTransform, Color, Easing, FontFamily, FontSource, ImageObject, Keyframe, Object, Shape,
    StyleFlags, StyledText, TextObject, TextRun, Track, Transform, Vec2,
};
pub use timeline::{Clip, Layer, Timeline};
