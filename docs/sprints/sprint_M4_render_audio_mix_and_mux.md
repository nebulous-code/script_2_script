# sprint_M4_render_audio_mix_and_mux.md

## Sprint Goal (M4)
Ensure **final MP4 includes audio**:
- background music (MP3) looped or bounded
- bounce SFX (OGG) at scheduled timestamps
- deterministic audio that matches the video timeline
- mux audio + video into a final output mp4

This is the key sprint that makes the output shareable.

---

## Scope

### In-scope
- Audio timeline representation:
  - one background track spec
  - list of sfx events with `time` + volume
- Deterministic event generation:
  - for now, events can come from demo logic (e.g., bouncing ball physics emits bounce times)
- ffmpeg-driven audio mixing for render mode:
  - generate an intermediate **audio.wav** (uncompressed PCM)
  - trim audio to `[start_time, end_time]` if needed
- mux step:
  - combine `video.mp4` + trimmed audio into `output.mp4`
- Preview vs Render semantics (explicit):
  - **Preview uses realtime audio via raylib (no ffmpeg / no wav generation)**
  - **Render uses ffmpeg offline mixing (wav intermediate)**
- Logging a warning if SFX event count is high (no hard cap)

### Out-of-scope
- Full Rust audio engine (future)
- Multiple background tracks, ducking, envelopes (future)
- Automatic collision triggers across arbitrary objects (future)
- Sample-accurate preview seeking guarantees (best-effort seeking is fine for alpha)

---

## Locked decisions (from review)

### Intermediate format
- Render-mode audio intermediate is **`audio.wav`**, then mux/encode to AAC into the final MP4.
  - Rationale: easiest to debug, fewer codec surprises in alpha.

### Start/end handling
- **Render mode:** render full timeline audio, then **trim** to `[start_time, end_time]`, then mux.
  - Rationale: simplest and least error-prone; correctness > speed in alpha render.
- **Preview mode:** do **not** generate wav or run ffmpeg.
  - Preview uses realtime raylib audio playback and starts the timeline clock at `start_time`.

### “Too many SFX events”
- No hard cap in alpha.
- If SFX events exceed a threshold, log a warning (e.g., `> 100`).
  - Rationale: this is a video tool, but a bouncing-ball demo may still generate dozens of events; warnings are better than errors.

---

## Preview vs Render semantics (must be implemented & documented)

### Preview mode (raylib realtime)
- No ffmpeg mixing. No wav generation. No trimming step.
- Timeline clock starts at `start_time` and advances by fixed dt (`1/fps`).
- Background music:
  - best effort: seek music stream to `start_time` if supported by the binding
  - otherwise play from 0 (document limitation) OR treat as an error based on caller policy
- SFX playback:
  - events are only fired for times within `[start_time, end_time]`
  - if using simulation-driven events (ball bounces), fast-forward the simulation state to `start_time` without rendering, then begin normal preview

### Render mode (ffmpeg offline)
- Deterministically collect SFX events while rendering the video frames.
- Produce `audio_full.wav` for `[0..duration]`.
- If `start_time/end_time` are not full:
  - trim `audio_full.wav` to `audio_clip.wav`
  - mux `video_clip.mp4` + `audio_clip.wav` into final output

---

## Tasks (Agent Checklist)

### Data model
- [ ] Define audio timeline types:
  - [ ] `MusicTrack { path, start, end, loop, volume }`
  - [ ] `SfxEvent { path, time, volume }`
- [ ] Define SFX event count warning:
  - [ ] if `events.len() > 100`, log a warning (do not fail)

### Event collection (render mode)
- [ ] Implement event collection for the bouncing ball example:
  - [ ] during deterministic render loop, detect wall collisions and record event time `t`
  - [ ] ensure corner bounces create 1 event not 2
  - [ ] record only events within `[start_time, end_time]` for the final segment render
    - (optional simplification) if generating full audio, record full and trim later

### ffmpeg audio render (offline)
- [ ] Implement ffmpeg audio render to **WAV**:
  - [ ] background mp3 is looped or bounded to match `duration`
  - [ ] delay each SFX by event time (ms) using `adelay`
  - [ ] mix background + sfx using `amix`
  - [ ] output to `audio_full.wav`
- [ ] Implement trimming step (if `start_time/end_time` are not the full range):
  - [ ] trim `audio_full.wav` → `audio_clip.wav` using `-ss` and `-t`/`-to`
  - [ ] ensure duration matches the video clip duration

### Video+Audio mux
- [ ] Implement mux:
  - [ ] mux `video_clip.mp4` + `audio_clip.wav` → `output.mp4`
  - [ ] encode audio to AAC during mux (or equivalent)
  - [ ] ensure `-shortest` is used to avoid trailing audio/video mismatch

### Single render command path
- [ ] Provide a single render command path:
  - [ ] `--render` produces `output.mp4` with audio included
  - [ ] optionally keep intermediates with `--keep-temp`

### Diagnostics
- [ ] On ffmpeg failure:
  - [ ] surface ffmpeg stderr in the returned error (do not fail silently)

---

## Deliverables
- `output.mp4` that includes:
  - background music throughout
  - bounce SFX aligned to collisions
  - correct clipping to `[start_time, end_time]`

---

## Acceptance Criteria
- [ ] Background audio is audible throughout the final mp4 segment.
- [ ] Bounce SFX occurs at correct times (visually aligned with bounces).
- [ ] Output duration matches `[start_time, end_time]`.
- [ ] Render pipeline is deterministic (same input -> same timestamps -> same output).
- [ ] Preview does not run ffmpeg or generate WAV files; it uses realtime audio playback.

---

## Risks / Notes
- ffmpeg filter graphs can get long if there are many SFX events; we log a warning if event count is high.
- MP3/OGG decoding is handled by ffmpeg for render mode.
- Preview seeking support depends on the raylib binding; treat seek failures as `Result` and let caller decide policy.

---

## Open Decisions
- None for M4 (intermediate format, start/end handling strategy, and SFX event policy are now locked in).
