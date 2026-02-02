# Sprint Goal (M2)

Add **preview mode** features:
- realtime playback in a raylib window
- audio preview:
  - loop background mp3
  - play bounce ogg on event (for now triggered by a simple demo event, not auto collisions)

This establishes the “interactive iteration loop.”

---

## Scope

### In-scope
- Preview runner:
  - `--start_time`, `--end_time`
  - play/pause optional (nice-to-have)
- Audio playback using raylib audio device:
  - background music stream must be updated each frame
  - sound effects play on demand
- Example demo that triggers sfx on a predictable schedule (e.g., every second)

### Out-of-scope
- Offline audio rendering (next sprint)
- Automatic collision/overlap event system
- Scrubbing/jumping UI (alpha will use start/end range instead)

---

## Locked decisions (from review)
- **Preview time advances using a fixed dt** (`dt = 1.0 / fps`), not wall-clock elapsed time.
  - This keeps preview behavior consistent even if:
    - the machine is slow
    - fps changes in config
    - future “sped up playback” is added
- **No scrubbing/jumping support in alpha**
  - Use `--start_time` and `--end_time` for precise ranges instead.
- **Missing audio assets return Result**
  - Audio initialization and asset loading return `Result` so callers can decide whether to hard-fail or degrade gracefully.

---

## Tasks
- [x] Implement CLI parsing (if not already):
  - [x] `--start_time`, `--end_time`
  - [x] `--preview` default mode (optional if preview is default)
- [x] Implement preview time stepping:
  - [x] set `dt = 1.0 / fps`
  - [x] each frame: `t = t + dt`
  - [x] stop at `end_time`
  - [x] note: preview may render slower than realtime on slow machines; correctness > wall clock
- [x] Add `AudioEngine` module returning Results:
  - [x] `AudioEngine::new(...) -> Result<AudioEngine, AudioError>`
  - [x] initializes audio device
  - [x] loads music from `assets/background.mp3`
  - [x] loads sfx from `assets/border.ogg`
  - [x] `start_background(looped: bool)`
  - [x] `update()` calls `UpdateMusicStream` every frame
  - [x] `play_bounce()` plays the sfx
  - [x] cleanup on drop
- [x] Wire preview renderer to audio:
  - [x] call `audio.update()` each frame
  - [x] trigger sfx via a deterministic test event:
    - [x] e.g. play once when integer seconds tick over
- [x] Add `examples/m2_preview_audio.rs`:
  - [x] shows visuals
  - [x] plays background music
  - [x] plays bounce sfx on schedule
- [x] Update README / examples:
  - [x] document fixed-dt preview
  - [x] document that audio init/load returns Result (caller chooses error handling policy)

---

## Deliverables
- `examples/m2_preview_audio.rs` demonstrating:
  - fixed dt time progression
  - background mp3 playback
  - repeated sfx playback

---

## Acceptance Criteria
- [x] Background MP3 loops and plays during preview (when assets exist).
- [x] SFX can play multiple times without crashing.
- [x] Preview respects `--start_time` and `--end_time`.
- [x] Preview time uses fixed dt (`1/fps`) and is deterministic given fps + start/end.
- [x] Missing/invalid audio assets cause `AudioEngine::new` to return `Err` (no hidden panics).

---

## Risks / Notes
- MP3 support depends on the raylib build; if mp3 is unsupported in a given environment, that should surface as an `Err`.
- Fixed-dt preview means wall-clock speed may not match realtime if rendering is slow (acceptable for alpha).

---

## Open Decisions
- None for M2 (time stepping, scrubbing, and missing-asset behavior are now locked in).
