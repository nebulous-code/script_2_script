# Sprint Goal (M0)

Create the minimal **declarative timeline** that can be sampled at time `t` (seconds) and rendered into a 2D frame:
- Timeline → Layers → Clips
- Clips have start/end times and a renderable Object
- Renderer samples active clips at `t` and draws them in z-order

This sprint should produce a working “hello timeline” example with shapes and images.

---

## Scope

### In-scope
- Core types: `Timeline`, `Layer`, `Clip`, `Object`
- Time: `f32 seconds`
- Layer ordering (z-order): **implicit insertion order by default**, with optional override
- Basic renderables:
  - `Shape::Circle`, `Shape::Rect` (at least)
  - `Image` (PNG/JPG)
- Basic transform (static):
  - position
  - opacity (even if constant for now)
- Rendering backend: raylib preview window (no audio yet)
- **Standardized coordinate system**: center-origin “graph coords” everywhere outside the raylib backend

### Out-of-scope
- Animation curves / easing (next sprint)
- ffmpeg output (later sprint)
- Audio (later sprint)
- Video clips (later sprint)
- Markdown text formatting (later sprint)

---

## Proposed Module Layout
- `src/lib.rs` (exports public API)
- `src/timeline/mod.rs`
  - `timeline.rs`, `layer.rs`, `clip.rs`
- `src/scene/mod.rs`
  - `object.rs`, `shape.rs`, `image.rs`, `transform.rs`
- `src/backend/raylib_preview.rs` (or `renderer/raylib.rs`)
  - owns all conversions between graph coords and raylib screen coords
- `examples/m0_hello_timeline.rs`

---

## Key Data Structures (target shape)

### Timeline
- `duration: f32`
- `fps: u32` (config value; sampling uses `t`)
- `layers: Vec<Layer>`

### Layer
- `name: String`
- `z_override: Option<i32>` *(optional override; default None)*
- `clips: Vec<Clip>`
- **insertion_index** is implied by `Timeline.layers` order (no field required; computed at runtime)

**Ordering rule (must be documented and stable):**
- Default draw order is layer insertion order.
- If `z_override` is set, use it to reposition the layer relative to others.
- Suggested implementation rule:
  - `effective_z = z_override.unwrap_or(insertion_index as i32)`
  - sort by `(effective_z, insertion_index)` so ties preserve insertion order

### Clip
- `start: f32`
- `end: f32`
- `object: Object`
- `transform: Transform` (static for M0)

### Object
- `Shape(ShapeObject)`
- `Image(ImageObject)`
- (Text/Audio/Video later)

### Transform (M0)
- `pos: Vec2` *(graph coords: origin at center, +y up)*
- `scale: Vec2` (optional; can default to (1,1))
- `rotation: f32` (optional; can default 0)
- `opacity: f32` (0..1)

---

## Coordinate System Standard (Graph Coords)
This is locked in for the project to avoid constant mental conversions.

### Graph coords (public API)
- origin `(0,0)` is **center of the canvas**
- +X is right, -X is left
- +Y is up, -Y is down
- top-right corner is `(width/2, height/2)`
- bottom-left is `(-width/2, -height/2)`

### Raylib screen coords (backend only)
Raylib uses top-left origin with +Y downward. The backend must convert:

- `screen_x = (width / 2) + x`
- `screen_y = (height / 2) - y`

**Rule:** No raylib coordinates leak out of the backend. All scene/timeline code uses graph coords only.

---

## Tasks
- [x] Create `Timeline/Layer/Clip` structs with **Result-based validation**:
  - [x] constructors/builders return `Result<_, Error>` (no panics for invalid user inputs)
  - [x] enforce `0 <= start < end <= duration`
  - [x] clips can overlap; ordering resolves by layer ordering rules
- [x] Implement `Timeline::sample(t: f32) -> SampledScene`
  - [x] selects active clips: `start <= t < end`
  - [x] orders layers by the defined implicit+override rule (stable sort)
- [x] Implement `Renderable` trait (or similar):
  - [x] `fn draw(&self, ctx: &mut DrawCtx, transform: &Transform) -> Result<()>`
- [x] Implement `Shape` drawing via raylib backend
  - [x] backend takes graph coords and converts internally
- [x] Implement `Image` loading + drawing via raylib backend
  - [x] cache textures so they load once (keyed by asset path)
  - [x] drawing uses graph coords (backend converts to screen coords)
- [x] Implement preview renderer:
  - [x] fixed realtime preview loop
  - [x] `t` advances by dt (based on fps or actual elapsed time)
  - [x] render all active clips each frame
- [x] Add `examples/m0_hello_timeline.rs`
  - [x] a circle and a logo image appear at different times/layers
  - [x] include at least one overlap moment to visually confirm layer ordering
- [x] Add minimal README snippet describing M0 example + graph coord convention

---

## Deliverables
- A runnable example that opens a window and plays a 5–10 second timeline with:
  - at least 1 image clip
  - at least 1 shape clip
  - visible layer ordering
  - consistent center-origin coordinates

---

## Acceptance Criteria
- [x] Running `cargo run --example m0_hello_timeline` opens a window and shows a deterministic sequence of visuals.
- [x] Timeline sampling is time-based (seconds), not frame-index-based internally.
- [x] Asset loading is not repeated every frame (basic caching exists).
- [x] Z-order is predictable and documented:
  - default insertion order + optional `z_override`
- [x] Public API never requires users to think in raylib coordinates.

---

## Risks / Notes
- Opacity compositing may require a render target or per-draw alpha support; if hard, allow opacity=1 only for M0 and keep the field for later.
- Image anchors aren’t required in M0, but image placement should be center-origin consistent.

---