# Rust Render (Alpha)

Early-stage Rust library for declarative, timeline-based 2D video rendering. M0 focuses on core timeline sampling and a raylib preview backend.

## M0 Preview Example

```bash
cargo run --example m0_hello_timeline
```

This opens a window and renders a short 8-second timeline with shapes and an image. The example uses `assets/logo.png`.

## M1 Animation Example

```bash
cargo run --example m1_animation
```

This demonstrates keyframed motion, easing, rotation, and opacity.

## M2 Preview + Audio Example

```bash
cargo run --example m2_preview_audio -- --start_time 0 --end_time 6
```

Preview time advances with fixed `dt = 1.0 / fps`, not wall-clock time. Audio init/loading returns `Result`, so missing assets surface as errors.

## M3 Render (Video Only)

```bash
cargo run --example m3_render_video -- --render --start_time 0 --end_time 6
```

This renders a video-only MP4 via ffmpeg using deterministic sampling.

## Coordinate System (Graph Coords)

All public APIs use center-origin graph coordinates:

- `(0, 0)` is the center of the canvas
- +X is right, -X is left
- +Y is up, -Y is down
- Top-right corner is `(width/2, height/2)`
- Bottom-left is `(-width/2, -height/2)`

The raylib backend converts graph coords to screen coords internally; raylib coordinates never appear in the public API.

## Dependencies

- `raylib` — preview window + drawing backend
- `anyhow` — error handling

## Project Layout (M0)

```
src/
  lib.rs
  timeline/
  scene/
  backend/
examples/
  m0_hello_timeline.rs
  m1_animation.rs
  m2_preview_audio.rs
  m3_render_video.rs
assets/
  logo.png
```

## Notes

- Timeline sampling is time-based (`f32` seconds), not frame-index-based.
- Layer ordering is stable: insertion order by default, with optional `z_override`.
