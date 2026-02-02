pub mod ffmpeg_video;
pub mod ffmpeg_audio;
pub mod video_clips;

pub use ffmpeg_video::FfmpegVideoEncoder;
pub use ffmpeg_audio::{mux_video_audio, render_audio_wav, trim_audio};
pub use video_clips::{build_base_video, ffprobe_metadata, normalize_if_needed, VideoMetadata};
