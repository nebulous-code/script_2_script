# Sprint Goal (M1)

Add **animation primitives** so clip transforms can vary over time:
- position, scale, rotation, opacity
- support at least:
  - linear interpolation
  - two easing functions:
    - `ease_in_out_quad`
    - `ease_out_cubic`

This turns the timeline into “motion graphics” rather than static clips.

---

## Scope

### In-scope
- A structured way to express time-varying properties using **keyframes**
- Interpolation + easing functions
- A clip-local time helper to make reusable animation authoring easier
- Update preview example(s) to demonstrate motion and easing

### Out-of-scope
- Advanced effects (blur, shadows)
- Physics simulation / collisions (later)
- Audio/video import (later)

---

## Approach

### Keyframe Tracks
Each transform property has a keyframe track:
- `Track<Vec2>` for **position** / **scale**
- `Track<f32>` for **rotation** / **opacity**

A keyframe is:
- `time: f32` (seconds)
- `value: T`

**Keyframe time rule:** keyframe times must be **strictly increasing**.  
No duplicates are allowed. If a “teleport” is desired, author it as `t` and `t + epsilon`.

### Easing stored on segments
Easing is modeled as a property of the **segment between two keyframes**.

Conceptually:
- Segment i is the interval `[key_i, key_{i+1}]`
- Segment i has an `Easing`

Practical representation (recommended for simple structs):
- store `easing_to_next: Easing` on `key_i`
- semantics: `key_i.easing_to_next` applies to the transition from `key_i` to `key_{i+1}`

This preserves the correct mental model (easing belongs to “the move between keyframes”) while keeping the data structure simple.

---

## How sampling works (plain explanation)

When sampling a track at time `t`:
1. Find the two keyframes that surround `t`:
   - `k0` at time `t0`
   - `k1` at time `t1`
2. Convert `t` into normalized progress between them:
   - `u = (t - t0) / (t1 - t0)`  where `u` is in `[0,1]`
3. Apply the segment’s easing:
   - `u2 = easing(u)`
4. Interpolate the value:
   - `value = lerp(k0.value, k1.value, u2)`

Outside the keyframe range:
- If `t` is before the first keyframe, hold the first value
- If `t` is after the last keyframe, hold the last value

---

## Tasks
- [ ] Define `Easing` enum + functions:
  - [ ] `Linear`
  - [ ] `EaseInOutQuad`
  - [ ] `EaseOutCubic`
- [ ] Implement generic `Track<T>` + `Keyframe<T>`
  - [ ] enforce strictly increasing keyframe times (return `Result`)
  - [ ] `sample(t) -> T`
  - [ ] clamp/hold behavior outside keyframe range
- [ ] Add `AnimatedTransform` with tracks:
  - [ ] position track
  - [ ] scale track
  - [ ] rotation track
  - [ ] opacity track
  - [ ] provide a “constant transform” constructor that builds a track with 1 keyframe
- [ ] Update `Clip` to support animated transforms:
  - [ ] simplest: `transform: AnimatedTransform` where defaults are constant tracks
- [ ] Add `Clip::local_time(t)` helpers:
  - [ ] `local_time(t) -> Option<f32>`: returns `Some(t-start)` if active else `None`
  - [ ] `clamped_local_time(t) -> f32`: returns clamped `0..(end-start)` for convenience (optional)
- [ ] Update renderer to sample the transform at global time `t`:
  - [ ] optionally use `clip.local_time(t)` for relative track authoring patterns
- [ ] Update example(s):
  - [ ] circle moves across screen using `EaseInOutQuad`
  - [ ] image rotates using `EaseOutCubic`
  - [ ] fade-in/out via opacity

---

## Deliverables
- A new example: `examples/m1_animation.rs` demonstrating:
  - keyframed motion
  - two easing functions
  - at least two animated properties (e.g., position + opacity)

---

## Acceptance Criteria
- [ ] Motion is deterministic and matches expected keyframe interpolation.
- [ ] Easing visibly differs from linear interpolation.
- [ ] Strict keyframe time validation returns `Err` on invalid keyframe sequences.
- [ ] `Clip::local_time` is usable for authoring reusable “clip recipes.”

---

## Risks / Notes
- Opacity may require per-draw alpha blending; implement the simplest approach raylib supports first.
- Floating precision: define a small epsilon constant for author-authored “teleports.”

---

## Open Decisions
- None for M1 (keyframe strictness, segment-based easing semantics, and local time helpers are now locked in).
