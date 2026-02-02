# Rust Render (Alpha)

Early-stage Rust library for declarative, timeline-based 2D video rendering. M0 focuses on core timeline sampling and a raylib preview backend.

## M0 Preview Example

```bash
cargo run --example m0_hello_timeline
```

This opens a window and renders a short 8-second timeline with shapes and an image. The example uses `assets/logo.png`.

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
assets/
  logo.png
```

## Notes

- Timeline sampling is time-based (`f32` seconds), not frame-index-based.
- Layer ordering is stable: insertion order by default, with optional `z_override`.
