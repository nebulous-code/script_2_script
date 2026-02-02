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

## Tasks (Agent Checklist)
- [ ] Implement CLI parsing (if not already):
  - [ ] `--start_time`, `--end_time`
  - [ ] `--preview` default mode (optional if preview is default)
- [ ] Implement preview time stepping:
  - [ ] set `dt = 1.0 / fps`
  - [ ] each frame: `t = t + dt`
  - [ ] stop at `end_time`
  - [ ] note: preview may render slower than realtime on slow machines; correctness > wall clock
- [ ] Add `AudioEngine` module returning Results:
  - [ ] `AudioEngine::new(...) -> Result<AudioEngine, AudioError>`
  - [ ] initializes audio device
  - [ ] loads music from `input/background.mp3`
  - [ ] loads sfx from `input/border.ogg`
  - [ ] `start_background(looped: bool)`
  - [ ] `update()` calls `UpdateMusicStream` every frame
  - [ ] `play_bounce()` plays the sfx
  - [ ] cleanup on drop
- [ ] Wire preview renderer to audio:
  - [ ] call `audio.update()` each frame
  - [ ] trigger sfx via a deterministic test event:
    - [ ] e.g. play once when integer seconds tick over
- [ ] Add `examples/m2_preview_audio.rs`:
  - [ ] shows visuals
  - [ ] plays background music
  - [ ] plays bounce sfx on schedule
- [ ] Update README / examples:
  - [ ] document fixed-dt preview
  - [ ] document that audio init/load returns Result (caller chooses error handling policy)

---

## Deliverables
- `examples/m2_preview_audio.rs` demonstrating:
  - fixed dt time progression
  - background mp3 playback
  - repeated sfx playback

---

## Acceptance Criteria
- [ ] Background MP3 loops and plays during preview (when assets exist).
- [ ] SFX can play multiple times without crashing.
- [ ] Preview respects `--start_time` and `--end_time`.
- [ ] Preview time uses fixed dt (`1/fps`) and is deterministic given fps + start/end.
- [ ] Missing/invalid audio assets cause `AudioEngine::new` to return `Err` (no hidden panics).

---

## Risks / Notes
- MP3 support depends on the raylib build; if mp3 is unsupported in a given environment, that should surface as an `Err`.
- Fixed-dt preview means wall-clock speed may not match realtime if rendering is slow (acceptable for alpha).

---

## Open Decisions
- None for M2 (time stepping, scrubbing, and missing-asset behavior are now locked in).
