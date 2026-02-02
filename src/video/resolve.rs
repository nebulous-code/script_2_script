use crate::video::VideoClip;

#[derive(Debug, Clone, PartialEq)]
pub struct VideoSegment {
    pub clip_index: usize,
    pub timeline_start: f32,
    pub timeline_end: f32,
    pub source_start: f32,
}

pub fn resolve_segments(clips: &[VideoClip]) -> Vec<VideoSegment> {
    let mut boundaries: Vec<f32> = clips
        .iter()
        .flat_map(|c| [c.start_time, c.end_time])
        .collect();
    boundaries.sort_by(|a, b| a.partial_cmp(b).unwrap());
    boundaries.dedup_by(|a, b| (*a - *b).abs() < f32::EPSILON);

    let mut segments = Vec::new();
    for w in boundaries.windows(2) {
        let t0 = w[0];
        let t1 = w[1];
        if t1 <= t0 {
            continue;
        }

        let mut chosen: Option<usize> = None;
        for (idx, clip) in clips.iter().enumerate() {
            if clip.start_time <= t0 && clip.end_time >= t1 {
                chosen = Some(idx);
            }
        }

        if let Some(clip_index) = chosen {
            let clip = &clips[clip_index];
            let offset = t0 - clip.start_time;
            let source_start = clip.trim_start.unwrap_or(0.0) + offset;
            segments.push(VideoSegment {
                clip_index,
                timeline_start: t0,
                timeline_end: t1,
                source_start,
            });
        }
    }

    segments
}
