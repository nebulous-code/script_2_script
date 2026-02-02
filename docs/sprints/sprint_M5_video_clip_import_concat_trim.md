# Sprint Goal (M5)
Support **video clip import** in alpha as “stock footage”:
- optional trim (start/end)
- concatenation and insertion into the timeline
- no transforms (no moving/scaling/opacity/rotation)

Implementation can be ffmpeg-driven.

---

## Scope

### In-scope
- VideoClip asset type:
  - `path`, `start_time`, `end_time`
  - optional `trim_start`, `trim_end`
- Ability to place clips in a timeline (insert into middle)
- Rendering approach:
  - simplest: pre-compose a “base video track” via ffmpeg concat/trim and treat it as background
  - alpha allows this as an implementation detail

### Out-of-scope
- Transformable video layers
- Arbitrary compositing of multiple videos (picture-in-picture) in alpha
- Playback speed changes

---

## Tasks (Agent Checklist)
- [ ] Define `VideoClip` timeline object and constraints:
  - [ ] clips cannot overlap (alpha constraint) OR define overlap rule (first wins)
- [ ] Implement ffmpeg concat pipeline:
  - [ ] generate concat list file (or use filter_complex)
  - [ ] apply trims if present
  - [ ] output `base_video.mp4` for the requested [start,end] window
- [ ] Integrate with renderer:
  - [ ] easiest alpha: if video clips are present, the renderer uses the composed base video as the first/background layer
  - [ ] overlay shapes/images/text (optional if feasible) OR defer overlays until post-alpha
- [ ] Add example:
  - [ ] uses two mp4 clips with a trim and an insert

---

## Deliverables
- A render that stitches multiple mp4 clips into one coherent sequence.

---

## Acceptance Criteria
- [ ] Two mp4 files can be trimmed and concatenated into a final mp4 at the correct times.
- [ ] Insertion into the middle works (timeline order is respected).
- [ ] Rendering does not require decoding video frames in Rust for alpha.

---

## Risks / Notes
- Concatenation requires compatible codecs/resolution; ffmpeg can re-encode to normalize but that costs time.
- Define whether alpha forces a single resolution (project config) and re-encodes clips to match.

---

## Open Decisions
- Are video clips required to match project resolution/fps, or do we auto-normalize (re-encode)?
- Can video clips overlap in alpha? If yes, what does overlap mean?
- Does alpha require overlays on top of video clips, or is stitching-only acceptable for M5?
