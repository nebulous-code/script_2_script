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

---

## Tasks (Agent Checklist)
- [ ] Implement CLI parsing (if not already):
  - [ ] `--start_time`, `--end_time`, `--preview` default
- [ ] Add `AudioEngine` module:
  - [ ] `init_audio_device`
  - [ ] load music stream from `input/background.mp3`
  - [ ] load sound from `input/border.ogg`
  - [ ] `update()` calls `UpdateMusicStream`
  - [ ] `play_sfx()` plays bounce sound
  - [ ] cleanup on drop
- [ ] Wire preview renderer to:
  - [ ] pace at target fps
  - [ ] advance `t` from start_time to end_time
  - [ ] call `audio.update()` each frame
- [ ] Add demo event scheduling:
  - [ ] `if floor(t) changes -> play bounce` (for testing)
- [ ] Update README / examples:
  - [ ] document that preview audio is realtime

---

## Deliverables
- `examples/m2_preview_audio.rs`:
  - shows visuals
  - plays background music
  - plays bounce sfx on schedule

---

## Acceptance Criteria
- [ ] Background MP3 loops and plays continuously during preview.
- [ ] SFX can play multiple times without crashing.
- [ ] Preview respects `--start_time` and `--end_time`.
- [ ] No need to write files; purely preview.

---

## Risks / Notes
- MP3 support depends on raylib build; ensure your raylib distribution supports it.
- Realtime preview is not deterministic w.r.t. wall clock—but the sampled timeline is deterministic.

---

## Open Decisions
- Should preview time advance using fixed dt (`1/fps`) or actual elapsed time?
- Should preview support scrubbing/jumping, or only start/end range for now?
- How should missing audio assets behave (warn & continue vs hard error)?
