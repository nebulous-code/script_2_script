```md
# design_doc.md — Raylib → FFmpeg video rendering prototype (Rust)

## Goal

Build a small Rust prototype that:
1) renders a simple Raylib animation (a ball bouncing in a window),  
2) captures each frame as raw RGBA pixels,  
3) streams frames into an `ffmpeg` subprocess via stdin,  
4) produces an `.mp4` video file.

MVP animation:
- a circle (“ball”) bounces off the edges of an `800x600` scene
- its color cycles smoothly through **Red -> Yellow -> Green -> Cyan -> Blue -> Magenta back** (full loop)
- target framerate: 30fps
- target duration: e.g. 60 seconds

Long-term direction:
- turn this into a reusable “video renderer” library where the *rendering* is decoupled from the *encoding*.

---

## What we learned from the C repo (tsoding)

The C example does two important things:
- renders into a `RenderTexture2D` each frame (offscreen surface), then
- reads pixels back from GPU to CPU (`LoadImageFromTexture`) and streams them to ffmpeg as raw `rgba`.

FFmpeg is invoked as a child process expecting:
- `-f rawvideo`
- `-pix_fmt rgba`
- `-s WxH`
- `-r FPS`
- `-i -` (stdin)

Then it encodes to `output.mp4` with x264/yuv420p.

The repo also flips frames vertically in code; it notes you could instead do `-vf vflip` in ffmpeg.

---

## Rust approach

### High-level architecture

Two modules:
- `renderer/` → Raylib animation and frame capture
- `encoder/`  → ffmpeg child process and stdin streaming

Keep the seam clean:
- renderer returns `Vec<u8>` RGBA buffer each frame
- encoder accepts `&[u8]` and writes exactly `width*height*4` bytes per frame

### Suggested crate layout

src/
main.rs
config.rs
renderer/
  mod.rs
  bouncing_ball.rs
encoder/
  mod.rs
  ffmpeg.rs

## Dependencies (Cargo.toml)

### Minimal
- `raylib` crate (Rust bindings for raylib; includes the low-level `raylib::ffi::LoadImageFromTexture` capture API) :contentReference[oaicite:0]{index=0}

### Highly recommended for ergonomics
- `anyhow` (easy error handling)
- `thiserror` (optional: structured errors)
- `clap` (optional: CLI args for width/height/fps/duration/output)
- `which` (optional: find ffmpeg / better error messages)

You do **not** need ffmpeg library bindings for this pipeline; spawning the ffmpeg binary is the simplest route.

---

## External requirements

Make sure to do pre-flight checks that ffmpeg is on PATH

- `ffmpeg` available on PATH
  - macOS: `brew install ffmpeg`
  - Windows: install ffmpeg and ensure `ffmpeg.exe` is on PATH

---

## MVP spec

### Config
- `width: u32 = 800`
- `height: u32 = 600`
- `fps: u32 = 30`
- `duration_secs: u32 = 60`
- `output_path: PathBuf = "output.mp4"`

### Visual design
- background: dark neutral (e.g. `0x181818`)
- ball radius: `height / 8`
- velocity: ~200 px/sec each axis
- color cycling:
  - either HSV hue rotate (simplest and smoothest)
  - or piecewise linear segments that explicitly pass through:
    - Red → Yellow → Green → Cyan → Blue → Magenta → Red

---

## Encoding pipeline details (FFmpeg)

Use `std::process::Command` with piped stdin:

Suggested args (note `-vf vflip` avoids manual flip and matches raylib GPU coordinate orientation):

```bash
ffmpeg -y -loglevel error \
  -f rawvideo -pix_fmt rgba -s 800x600 -r 30 -i - \
  -vf vflip \
  -c:v libx264 -pix_fmt yuv420p -crf 18 \
  output.mp4
````

Notes:

* `-crf 18` is good quality; increase to 22–28 for smaller files.
* `-pix_fmt yuv420p` improves compatibility with many players.
* You can drop audio args entirely for MVP.

---

## Renderer details (Raylib)

Render strategy:

* create window + `RenderTexture2D`
* each frame:

  * draw into the render texture
  * read pixels: `raylib::ffi::LoadImageFromTexture(render_texture.texture)`
  * get `image.data` pointer (RGBA32)
  * copy to a `Vec<u8>` sized `width*height*4`
  * unload/free image

Raylib provides a `LoadImageFromTexture` function (Rust bindings expose it via the `ffi` module) ([Docs.rs][1])
Raylib also provides “textures_to_image” example showing the same concept in C. ([raylib][2])

---

## Pseudocode workflow

### main loop (deterministic offline render)

```text
config = {w,h,fps,duration,output}
total_frames = fps * duration

encoder = FfmpegEncoder::start(config)

raylib = init_window(w,h)
render_tex = load_render_texture(w,h)

state = BouncingBallState::new(w,h)

for frame_index in 0..total_frames:
    t = frame_index / fps
    state.step(dt = 1/fps)

    begin_texture_mode(render_tex)
        clear_background(bg)
        draw_ball(state.position, state.radius, state.color_at(t))
    end_texture_mode()

    // capture RGBA pixels
    image = LoadImageFromTexture(render_tex.texture)
    bytes = copy(image.data, w*h*4)
    unload_image(image)

    encoder.write_frame(bytes)

encoder.finish()
close_window()
```

### encoder module pseudocode

```text
start():
    spawn ffmpeg with stdin piped
    store Child + ChildStdin

write_frame(frame_rgba):
    assert len == w*h*4
    stdin.write_all(frame_rgba)

finish():
    drop stdin (close)
    wait on child
    error if exit status != 0
```

---

## Implementation tasks

### Phase 0 — Repo bootstrap

* [ ] Create Rust crate: `cargo new raylib_video --bin`
* [ ] Add dependencies: `raylib`, `anyhow` (and optionally `clap`)
* [ ] Add `config.rs` with width/height/fps/duration/output fields
* [ ] Add `.gitignore` for `output.mp4`

### Phase 1 — FFmpeg encoder module (no raylib yet)

* [ ] Implement `encoder::ffmpeg::FfmpegEncoder`

  * [ ] `start(width, height, fps, output_path) -> Result<Self>`
  * [ ] `write_frame(&mut self, rgba: &[u8]) -> Result<()>`
  * [ ] `finish(self) -> Result<()>`
* [ ] Add guardrails:

  * [ ] validate `rgba.len() == (w*h*4)`
  * [ ] detect missing `ffmpeg` and return a helpful error
* [ ] Write a quick unit-ish test binary path:

  * [ ] generate 60 frames of solid colors in memory and encode a video
  * [ ] confirm ffmpeg exit code is 0

### Phase 2 — Raylib render + capture

* [ ] Implement `renderer::bouncing_ball::BouncingBallRenderer`

  * [ ] `new(width, height) -> Self`
  * [ ] `step(dt)`: updates position + bounce
  * [ ] `color_at(t)`: returns RGBA cycling through CMY and RGB points
  * [ ] `draw(&mut rl, &thread, &render_tex)`: draws into texture
* [ ] Implement `capture_rgba(render_texture) -> Vec<u8>`

  * [ ] call `raylib::ffi::LoadImageFromTexture(...)` ([Docs.rs][1])
  * [ ] `unsafe` copy from `image.data` pointer into a Rust-owned `Vec<u8>`
  * [ ] free image
* [ ] Verify the produced mp4 is correct orientation:

  * [ ] if upside down, keep `-vf vflip` in ffmpeg args

### Phase 3 — Wire it together

* [ ] In `main.rs`, build the deterministic frame loop:

  * [ ] fixed timestep `dt = 1.0/fps`
  * [ ] loop exactly `fps*duration` frames (ignore realtime)
* [ ] Produce `output.mp4` in project root
* [ ] Add a `README.md` section “How to run”:

  * [ ] `cargo run --release`
  * [ ] mention `ffmpeg` requirement

### Phase 4 — Quality + DX

* [ ] CLI flags (optional): `--fps`, `--duration`, `--out`, `--size 800x600`, `--crf`
* [ ] Add progress output: “frame i / total”
* [ ] Add a “headless” flag idea (future): window hidden/minimized or platform-specific headless contexts

---

## Color cycling algorithm (concrete)

Simple deterministic “keyframe” cycling through:
`Red -> Yellow -> Green -> Cyan -> Blue -> Magenta -> Red`

Represent colors as floats [0,1], linear interpolate between each segment:

Segments (each 1/6 of the loop):

1. R(1,0,0) → Y(1,1,0)
2. Y(1,1,0) → G(0,1,0)
3. G(0,1,0) → C(0,1,1)
4. C(0,1,1) → B(0,0,1)
5. B(0,0,1) → M(1,0,1)
6. M(1,0,1) → R(1,0,0)

Implementation:

* `phase = (t * speed) % 1.0`
* `seg = floor(phase * 6)`
* `u = fract(phase * 6)`
* `color = lerp(colors[seg], colors[seg+1], u)`
* Convert to raylib `Color { r,g,b,a }` as u8.

---

## Known pitfalls + mitigations

* **Orientation flip**: GPU readback is often upside down.

  * Use `-vf vflip` in ffmpeg command (recommended) rather than flipping in Rust.
* **Performance**: `LoadImageFromTexture` per frame is not cheap.

  * OK for MVP; later optimize by reusing buffers, avoiding allocations, or lower-level pixel reads.
* **Exact byte count**: ffmpeg rawvideo expects exact frame size.

  * Always write `width*height*4` bytes per frame.
* **Pixel format mismatch**: ensure you send RGBA (not BGRA).

  * Keep `-pix_fmt rgba` consistent with raylib capture path.

---

## “Definition of Done” for MVP

* [ ] Running `cargo run --release` creates `output.mp4`
* [ ] Video plays correctly:

  * [ ] ball bounces at correct speed
  * [ ] colors cycle smoothly through RGB/CMY
  * [ ] no upside-down frames
  * [ ] correct duration (`duration_secs`) and fps

---

## Next steps after MVP

* Add “render scripts” (timeline-based animation primitives)
* Allow rendering without showing a window (headless contexts)
* Support other encoders/containers (gif, webm)
* Add optional audio track muxing
* Build a library API (`Renderer` trait + `Encoder` trait)

```

If you want, I can also generate a **starter Rust repo skeleton** (folders + stub modules + TODOs) matching this plan, so your Codex agent can start filling in real code immediately.
::contentReference[oaicite:4]{index=4}
```

[1]: https://docs.rs/raylib/latest/raylib/ffi/fn.LoadImageFromTexture.html?utm_source=chatgpt.com "LoadImageFromTexture in raylib::ffi - Rust"
[2]: https://www.raylib.com/examples/textures/loader.html?name=textures_to_image&utm_source=chatgpt.com "raylib [textures] example - texture to image"
