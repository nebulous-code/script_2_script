# sprint_M5_video_clip_import_concat_trim.md

## Sprint Goal (M5)
Support **video clip import** in alpha as “stock footage”:
- optional trim (start/end)
- concatenation and insertion into the timeline
- overlap handling with a simple rule (“newest stomps”)
- no transforms (no moving/scaling/opacity/rotation)
- stitching-only (no overlays on top of video)

Implementation is ffmpeg-driven.

---

## Scope

### In-scope
- `VideoClip` asset type:
  - `path`, `start_time`, `end_time`
  - optional `trim_start`, `trim_end` (source trim)
- Ability to place clips in a timeline (insert into middle)
- Overlap behavior:
  - overlaps allowed
  - latest-declared clip wins (“stomps” older clips) for overlapping time ranges
- Resolution/FPS normalization:
  - clips may be auto-normalized to project config using ffmpeg when needed
- Rendering approach:
  - compose a “base video track” via ffmpeg into a single MP4 that matches project settings
  - alpha does not require decoding video frames in Rust

### Out-of-scope
- Transformable video layers (move/scale/rotate/opacity)
- Picture-in-picture / multi-video compositing
- Speed changes / time scaling
- Overlays (shapes/images/text) on top of video in alpha
- Using clip audio as part of the final audio mix (M4 handles audio timeline)

---

## Locked decisions (from review)

### Normalization (re-encode) policy
- Use **ffprobe** to inspect each input video clip.
- If a clip matches project `width/height/fps` requirements, use it as-is.
- If it does not match, transcode it to a normalized intermediate matching the project config.
- Alpha may ignore/strip embedded video audio during normalization.

### Overlap handling
- Overlapping clips are allowed.
- Overlap resolution is simple: **newest clip stomps older clips**.
  - “Newest” is defined as later-declared/later-added in the script/timeline.
- Implementation should resolve overlaps into a final set of non-overlapping segments before ffmpeg stitching.

### Stitching-only
- Alpha video import is stitching-only.
- No overlays on top of the composed base video in M5.

---

## Tasks (Agent Checklist)
- [ ] Define `VideoClip` object:
  - [ ] `path: PathBuf`
  - [ ] `start_time: f32`, `end_time: f32` (timeline placement)
  - [ ] `trim_start: Option<f32>`, `trim_end: Option<f32>` (source trimming)
  - [ ] validation returns `Result` (times, trim bounds, existence)
- [ ] Implement ffprobe inspection:
  - [ ] read width/height/fps (and optionally pix_fmt)
  - [ ] return structured metadata
- [ ] Implement “normalize if needed” step:
  - [ ] if mismatch with project config, transcode to normalized intermediate
  - [ ] normalized target should match:
    - resolution: project width/height
    - fps: project fps
  - [ ] strip or ignore clip audio (alpha)
- [ ] Implement overlap resolution (“newest stomps”):
  - [ ] input: list of `VideoClip`s (declared order matters)
  - [ ] output: ordered list of non-overlapping segments covering only times where video exists
  - [ ] each output segment points to exactly one source clip + its source time mapping
- [ ] Implement stitching/concat pipeline:
  - [ ] for each resolved segment:
    - apply source trim if needed
    - ensure normalized format
  - [ ] concatenate segments into `base_video.mp4`
- [ ] Integrate into render flow:
  - [ ] if video clips exist, generate `base_video.mp4` as the visual output for M5
  - [ ] ensure it matches project resolution/fps
- [ ] Add example:
  - [ ] uses two mp4 clips
  - [ ] includes one overlap case demonstrating stomping behavior
  - [ ] includes one trim and one insertion into the middle

---

## Deliverables
- A render that stitches multiple mp4 clips into one coherent sequence matching project resolution/fps.
- Overlaps are handled without error using the “newest stomps” rule.

---

## Acceptance Criteria
- [ ] Two mp4 files can be trimmed and concatenated into a final mp4 at the correct times.
- [ ] Insertion into the middle works (timeline order is respected).
- [ ] Overlapping clips do not error:
  - later-declared clip replaces earlier clip over the overlapping time interval.
- [ ] If an input clip mismatches project config, it is auto-normalized (re-encoded) to match.
- [ ] If a clip already matches config, it is not needlessly re-encoded.
- [ ] Implementation does not require decoding video frames in Rust.

---

## Risks / Notes
- Concatenation requires compatible formats; normalization solves this but adds encode time.
- Define whether aspect ratio is preserved (letterbox) vs stretched; alpha can choose “stretch to fit” or “letterbox” and document it.

---

## Open Decisions
- None for M5 (normalization strategy, overlap semantics, and stitching-only scope are now locked in).
