# Sprint Goal (M4)
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
- ffmpeg-driven audio mixing:
  - generate `audio.m4a` (or `audio.wav`)
- mux step:
  - combine `video.mp4` + `audio` into `output.mp4`

### Out-of-scope
- Full Rust audio engine (future)
- Multiple background tracks, ducking, envelopes (future)
- Automatic collision triggers across arbitrary objects (future)

---

## Tasks (Agent Checklist)
- [ ] Define audio timeline types:
  - [ ] `MusicTrack { path, start, end, loop, volume }`
  - [ ] `SfxEvent { path, time, volume }`
- [ ] Implement event collection for the bouncing ball example:
  - [ ] during deterministic render loop, detect wall collisions and record event time `t`
  - [ ] ensure corner bounces create 1 event not 2
- [ ] Implement ffmpeg audio render:
  - [ ] loop background to match duration (or `-stream_loop -1` + `-t duration`)
  - [ ] delay each SFX by event time (ms) using `adelay`
  - [ ] mix background + sfx using `amix`
  - [ ] output to `audio.m4a` (AAC) OR `audio.wav`
- [ ] Implement mux:
  - [ ] `ffmpeg -i video.mp4 -i audio.m4a -c:v copy -c:a aac -shortest output.mp4`
- [ ] Provide a single render command path:
  - [ ] `--render` produces `output.mp4` with audio included
  - [ ] optionally keep intermediates with `--keep-temp`

---

## Deliverables
- `output.mp4` that includes the background track + bounce sounds.

---

## Acceptance Criteria
- [ ] Background audio is audible throughout the final mp4.
- [ ] Bounce SFX occurs at correct times (visually aligned with bounces).
- [ ] Output duration matches `[start_time, end_time]`.
- [ ] The pipeline is deterministic (same input -> same timestamps -> same output).

---

## Risks / Notes
- ffmpeg filter graphs can get long if there are many SFX events.
- MP3 decoding + OGG decoding is handled by ffmpeg (good).
- Start/end time implies audio must also be cropped to the same range.

---

## Open Decisions
- Do we generate `audio.wav` then mux, or generate `audio.m4a` directly?
- How do we handle `--start_time` (seek/crop) for audio:
  - shift event times and trim background, or render full and then trim?
- What is the max number of SFX events we expect in alpha before filter graphs become unwieldy?
