# Sprint Goal (M3)
Add **render mode** that outputs **video-only MP4** via ffmpeg:
- deterministic frame generation
- stream RGBA frames to ffmpeg via stdin
- no audio in this sprint (M4 adds audio)

---

## Scope

### In-scope
- Render runner:
  - `--render`
  - `--start_time`, `--end_time`
- ffmpeg encoder:
  - spawn ffmpeg
  - write raw RGBA frames
  - produce `video.mp4`
- Validation:
  - ffmpeg exists on PATH
  - output directory valid

### Out-of-scope
- Audio muxing/mixing (next sprint)
- Video clip import (later)
- Headless preview (non-goal)

---

## Tasks (Agent Checklist)
- [ ] Implement/confirm deterministic render loop:
  - [ ] `frames = floor((end-start)*fps)`
  - [ ] for each `i`: `t = start + i/fps`
  - [ ] sample timeline at `t` and render to offscreen target
- [ ] Capture RGBA buffer per frame
  - [ ] ensure correct pixel format RGBA
  - [ ] handle vertical flip (prefer `-vf vflip` in ffmpeg args)
- [ ] Implement `FfmpegVideoEncoder`:
  - [ ] args: width/height/fps/out_path
  - [ ] write frames (exact byte size)
  - [ ] finalize process + error handling
- [ ] Add `examples/m3_render_video.rs`:
  - [ ] renders a short scene to mp4

---

## Deliverables
- A deterministic MP4 video output without audio.

---

## Acceptance Criteria
- [ ] `cargo run --example m3_render_video -- --render` produces `video.mp4`.
- [ ] Output plays correctly and is not upside down.
- [ ] Start/end time flags produce clipped output length.
- [ ] Render speed can be faster than realtime (not tied to wall clock).

---

## Risks / Notes
- ffmpeg arguments must match pixel format exactly.
- Some players require `-pix_fmt yuv420p` for compatibility.

---

## Open Decisions
- Output naming scheme (`video.mp4` vs user-provided output path)?
- Should render mode always create temp intermediates, or allow single-pass later?
- Error UX: show ffmpeg stderr on failure or keep it quiet?
