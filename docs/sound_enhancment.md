# design_doc.md — Add preview audio (background MP3 + bounce OGG)

## Goal

Extend the current prototype so that, while the window is running:

- `assets/background.mp3` plays continuously (looped) as background music
- `assets/border.ogg` plays once whenever the ball collides with a wall (left/right/top/bottom)

**Important scope note:** this phase is about **preview audio** (hearing it while the app runs).  
It does **not** embed audio into the encoded MP4 yet. Long-term, we’ll add an “audio track export + mux” step.

---

## Current repo snapshot (what exists today)

- Rendering:
  - `src/renderer/mod.rs` creates a raylib window + render texture
  - `render_frame()` updates ball physics, draws to render texture, captures RGBA via `LoadImageFromTexture`
- Encoding:
  - `src/encoder/ffmpeg.rs` streams raw RGBA frames to an `ffmpeg` child process
- Physics:
  - `src/renderer/bouncing_ball.rs` updates position/velocity and bounces off edges

---

## Audio integration strategy

Raylib audio has two key concepts:

- **Music** (streaming): best for MP3 background tracks  
  You must call `UpdateMusicStream(music)` every frame.
- **Sound** (loaded into memory): best for short OGG/WAV effects  
  Call `PlaySound(sound)` on events.

We’ll add a small audio manager that:
- initializes the audio device once
- loads background music as `Music`
- loads bounce effect as `Sound`
- plays/updates music continuously
- plays bounce sound on collision events
- cleans up on drop / shutdown

---

## Constraints / gotchas

### 1) Your loop is currently “offline-fast” (no realtime pacing)
Right now the `for frame_index in 0..total_frames` loop will run as fast as it can.  
Audio playback won’t behave meaningfully unless time actually passes.

✅ For this proof-of-concept, we will add a **preview mode** that runs at realtime FPS:
- call `set_target_fps(fps)` and perform a normal `begin_drawing()/end_drawing()` per frame
- this causes raylib to process OS events and pace the loop

Later we can support:
- `--preview` (realtime) vs `--render` (as fast as possible)
- and audio export/mux for `--render`

---

## Proposed changes (high-level)

1) **Add audio module**
   - `src/audio/mod.rs`
   - `AudioEngine` struct:
     - `init()`
     - `load(background_mp3, bounce_ogg)`
     - `start_background(looped)`
     - `update()` (per frame)
     - `play_bounce()`

2) **Modify ball physics to report collision**
   - Change `BouncingBall::step(dt)` to return whether a bounce happened this step

3) **Wire audio into renderer loop**
   - In `BouncingBallRenderer::new()`: init audio + load assets
   - In `render_frame()`: after stepping physics, if bounce -> play sound; every frame -> update music stream
   - In the preview path: draw to the screen (not just to the render texture) so raylib can pace frames

4) **Config**
   - Add:
     - `enable_audio: bool` (default true)
     - `preview_realtime: bool` (default true for now)
     - audio asset paths (default `assets/background.mp3`, `assets/border.ogg`)

---

## Detailed implementation plan (Codex checklist)

### Phase A — Add config fields

**File:** `src/config.rs`

Add fields:
- `pub enable_audio: bool`
- `pub preview_realtime: bool`
- `pub background_music_path: PathBuf`
- `pub bounce_sound_path: PathBuf`

Defaults:
- `enable_audio: true`
- `preview_realtime: true`
- `background_music_path: PathBuf::from("assets/background.mp3")`
- `bounce_sound_path: PathBuf::from("assets/border.ogg")`

Add helper:
- `pub fn validate_assets(&self) -> Vec<String>` returning warnings for missing files

(If the `assets/` folder isn’t present in your zip right now, create it and add the files there.)

---

### Phase B — Add bounce reporting

**File:** `src/renderer/bouncing_ball.rs`

Change:

```rust
pub fn step(&mut self, dt: f32)
````

to:

```rust
pub struct BounceEvent {
    pub bounced: bool,
    pub x: bool,
    pub y: bool,
}

pub fn step(&mut self, dt: f32) -> BounceEvent
```

Implementation behavior:

* Start with `let mut ev = BounceEvent { bounced:false, x:false, y:false };`
* Whenever you clamp position and invert velocity on X wall: set `ev.bounced=true; ev.x=true;`
* Same for Y wall
* Return `ev`

Corner case: if both X and Y collide in one step, you’ll get `x=true && y=true`; play the sound once.

---

### Phase C — Add audio module

**New files:**

* `src/audio/mod.rs`

Exports:

* `pub struct AudioEngine { ... }`

Design:

* `AudioEngine::new(config: &Config) -> Result<Option<AudioEngine>>`

  * returns `Ok(None)` if `enable_audio=false` OR assets missing
* `fn update(&mut self)` (called once per frame)
* `fn play_bounce(&mut self)`

#### Raylib API approach

You have two implementation options:

**Option 1 (preferred): use raylib crate safe wrappers (if available)**

* Look for types/functions like:

  * `RaylibAudio::init_audio_device()`
  * `Music` loading from file (MP3)
  * `Sound` loading from file (OGG)
  * `music.play()`, `music.update()`, `sound.play()`, `set_volume(...)`

**Option 2 (always works): use the C FFI**
Your project already uses `raylib::ffi` for frame capture; do the same for audio.

FFI functions to use (names match raylib C API):

* `InitAudioDevice()`
* `LoadMusicStream(path)`
* `PlayMusicStream(music)`
* `UpdateMusicStream(music)`
* `SetMusicVolume(music, volume)`
* `LoadSound(path)`
* `PlaySound(sound)`
* `SetSoundVolume(sound, volume)`
* `UnloadSound(sound)`
* `UnloadMusicStream(music)`
* `CloseAudioDevice()`

**Memory management**

* Add `impl Drop for AudioEngine` that unloads sound/music and closes audio device.

**Volumes (starting values)**

* background volume: `0.25`
* bounce volume: `0.7`

**Looping music**

* In C raylib, `Music` has a `looping` field. If the safe wrapper exposes it, set it.
* If using FFI, set `music.looping = true;` (confirm the struct field name in the generated bindings).

---

### Phase D — Wire audio into renderer

**File:** `src/renderer/mod.rs`

Add:

* `mod audio;` at crate root (in `src/main.rs` you’ll add `mod audio;`)
* In `BouncingBallRenderer` struct:

  * `audio: Option<AudioEngine>`

In `new(config)`:

* if `config.preview_realtime` is true:

  * call `self.rl.set_target_fps(config.fps as u32 or i32 depending on API)`
* init `audio = AudioEngine::new(config)?`

In `render_frame(...)`:

* call `let bounce = self.state.step(dt);`
* if `bounce.bounced` and `audio.is_some()`:

  * `audio.play_bounce()`
* every frame (if audio):

  * `audio.update()` (to keep MP3 streaming)

#### Add a screen draw pass (preview)

Right now you only render to the texture. For preview pacing and visual sanity, add:

```rust
{
    let mut d = self.rl.begin_drawing(&self.thread);
    d.clear_background(self.bg);
    let color = self.state.color_at(t_norm);
    d.draw_circle_v(self.state.position, self.state.radius, color);
}
```

This ensures:

* OS events are processed
* target FPS pacing works
* music stream timing behaves correctly

(You can keep the render-to-texture draw for capture; yes, it means drawing twice, but it’s fine for a POC.)

---

### Phase E — Minimal acceptance tests

Manual test checklist:

1. **Audio device init**

* Run: `cargo run`
* Expect: no panic; if audio files missing, print a warning and run silently without audio

2. **Background music**

* Confirm `background.mp3` starts immediately and loops

3. **Bounce sfx**

* Each time the ball hits a wall, hear a short bounce sound
* Corner hit should play **one** bounce sound (not two)

4. **No “machine-gun bounce”**

* Bounce should only trigger on the collision frame, not multiple frames in a row

5. **Output video still generated**

* Confirm MP4 output still renders as before (audio not embedded yet)

---

## Pseudocode (end-to-end)

```text
main():
  config = Config::default()
  renderer = BouncingBallRenderer::new(config)
  encoder = FfmpegEncoder::start(config)

  for frame_index in 0..total_frames:
    if renderer.window_should_close(): break
    frame_rgba = renderer.render_frame(frame_index, total_frames)
    encoder.write_frame(frame_rgba)

  encoder.finish()
  renderer drops -> audio cleaned up
```

Inside renderer:

```text
render_frame(i,total):
  bounce = ball.step(dt)

  draw ball to render_texture (for capture)
  draw ball to screen (for preview pacing)

  if audio:
    audio.update()
    if bounce: audio.play_bounce()

  capture RGBA
  return bytes
```

---

## Future (out of scope, but directionally important)

To actually embed audio into the MP4 deterministically:

* record `BounceEvent`s with timestamps
* synthesize or place samples onto an audio buffer
* write `audio.wav`
* mux with ffmpeg: `ffmpeg -i video.mp4 -i audio.wav -c:v copy -c:a aac out.mp4`

This avoids realtime dependencies and makes your “video programming language” deterministic.

---
