# Sprint Goal (M3)

Add **render mode** that outputs **video-only MP4** via ffmpeg:
- deterministic frame generation
- stream RGBA frames to ffmpeg via stdin
- produce an MP4 video file (no audio in this sprint)

---

## Scope

### In-scope
- Render runner:
  - `--render`
  - `--start_time`, `--end_time`
- ffmpeg encoder:
  - spawn ffmpeg
  - write raw RGBA frames
  - produce MP4 output
- Output naming rules (script-based default)
- Temp intermediates control (off by default)
- Validation:
  - ffmpeg exists on PATH
  - output directory valid
- Error UX:
  - surface ffmpeg stderr clearly on failure

### Out-of-scope
- Audio muxing/mixing (next sprint)
- Video clip import (later sprint)
- Headless preview (non-goal)

---

## Locked decisions (from review)

### Output naming scheme
- Default output name should be derived from the “script” `.rs` filename:
  - rendering `my_scene.rs` defaults to output `my_scene.mp4`
- If an explicit output path/name is provided:
  - via CLI (preferred override), or
  - via script config (prototype behavior),
  then the explicit value wins.

> Note: the implementation must define what “script file” means for alpha.
> If render is triggered from an example binary, the binary name can be used as the fallback.

### Temp intermediates
- Temp intermediates are controlled by a flag and are **off by default**.
- Enable via:
  - CLI flag (e.g., `--keep-temp`), and/or
  - config in the script
- When off, the renderer should clean up intermediate files on success (and ideally also on failure unless they are needed for diagnostics).

### ffmpeg stderr visibility
- Always show ffmpeg stderr on failure (surface errors directly).
- Do not fail silently.

---

## Tasks (Agent Checklist)
- [x] Implement/confirm deterministic render loop:
  - [x] `frames = floor((end-start)*fps)`
  - [x] for each `i`: `t = start + i/fps`
  - [x] sample timeline at `t` and render to an offscreen buffer
- [x] Capture RGBA buffer per frame
  - [x] ensure correct pixel format RGBA
  - [x] handle vertical flip (either in capture or via ffmpeg args, e.g. `-vf vflip`)
- [x] Implement `FfmpegVideoEncoder`:
  - [x] args: width/height/fps/output_path
  - [x] spawn ffmpeg process
  - [x] write each frame (exact byte size)
  - [x] finalize process, check exit code
  - [x] on failure: emit stderr in error return
- [x] Implement output path resolution:
  - [x] If CLI output is set: use it
  - [x] Else if script config output is set: use it
  - [x] Else default to `<script_stem>.mp4`
- [x] Implement temp intermediate behavior:
  - [x] define intermediate paths (e.g., `video_only.mp4` in a temp dir)
  - [x] `--keep-temp` retains them; default removes them on success
- [x] Add `examples/m3_render_video.rs` (or equivalent render entry point):
  - Note: Demo code should include clear, beginner-friendly comments explaining what each section does.
  - [x] renders a short scene to mp4 using render mode

---

## Deliverables
- Deterministic MP4 video output (video-only).

---

## Acceptance Criteria
- [x] Running render produces an MP4 video (no audio) that plays correctly.
- [x] Output default name follows the script stem:
  - `my_scene.rs` → `my_scene.mp4` when no output is provided.
- [x] `--start_time` and `--end_time` correctly clip render output.
- [x] `--keep-temp` (or config equivalent) retains intermediates; default cleans them up.
- [x] On ffmpeg failure, stderr is surfaced and the process returns a failure Result (no silent success).

---

## Risks / Notes
- ffmpeg arguments must match pixel format exactly.
- Some players require `-pix_fmt yuv420p` for compatibility.
- Clarify “script name” fallback for alpha if the render entry point is not literally executing a `.rs` file.

---

## Open Decisions
- None for M3 (output naming, temp intermediates flag, and ffmpeg error visibility are now locked in).
