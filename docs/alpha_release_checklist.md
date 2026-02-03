# Alpha Release Checklist (script2script)

This document consolidates the current “alpha readiness” work based on:
- the current sprint plan (M0–M6)
- our goals for long-term ergonomics and maintainability
- the additional code-review findings you pasted

The intent is to give a **high-context, actionable** list of remaining gaps to close before publishing to crates.io. Items are grouped by priority.

---

## What “alpha complete” means (definition of done)

Alpha is “complete” when a new user can:
- define a simple timeline with images/shapes/text (markdown subset) and audio events,
- preview a selected time window in a raylib window,
- render a selected time window to an **MP4 with audio included** (background + SFX),
- optionally import and stitch external video clips (stitching-only)
- get clear, actionable errors when prerequisites (like ffmpeg) are missing.

Alpha does **not** mean feature-complete (no full markdown, no video transforms, no advanced audio mixing). It does mean **deterministic**, **documented**, and **safe (no panics for normal user mistakes)**.

---

## Priority 0 (release blockers)

### 1) Fix offline SFX correctness when events reference different audio files
Right now the audio render path behaves as if there is only one SFX asset (it uses the first SFX file path for all events). 
This is a correctness bug: as soon as the public API allows `SfxEvent { path, time, volume }`, users will expect the chosen path to matter. The alpha fix can stay simple while being correct: **group SFX events by `path`**, add one ffmpeg input per unique path, apply event-specific `adelay`/volume per group, then mix the submixes with background via `amix`.

### 2) Make video clip trims accurate (avoid keyframe-snapped cuts when stream-copying)
Some segment trims are currently performed with `-ss` before `-i` combined with `-c copy`, which is fast but often **inaccurate** (snaps to keyframes). In a production-style tool, that’s a “why is this clip late/early?” footgun. For alpha, pick one deterministic policy and document it:
- **Accurate policy:** use `-ss` after `-i` and re-encode for trimmed segments, at least when the user requests a trim.

### 3) Remove panics from normal runtime paths (library-grade behavior)
A crates.io library should not panic for expected user mistakes like “forgot to preload an image” or “cache miss.” Replace panicking cache access (e.g., `expect("texture cache missing")`) with a `Result` (or make the panicking method private and route all public calls through fallible API). This matters for both maintainability (fewer bug reports) and ergonomics (users can handle errors cleanly in Rust).

### 4) Ensure preview start/end does not require offline audio rendering
The intent is preview should never depend on “generate full WAV then trim” just to see visuals. Preview audio should be **raylib realtime**, starting the timeline clock at `start_time` and playing only events within the requested range. If background music seeking is supported, seek; if not, either best-effort or return an error that the caller can choose to treat as fatal. This keeps iteration latency low for long-form projects.

### 5) Fix text wrapping edge case where long tokens overflow the bounding box
Alpha markdown credits often contain URLs, long names, file paths, or “no spaces” tokens. Word-wrap-only will overflow unless you implement a fallback. Make sure the “character wrap fallback for tokens longer than max width” is actually reachable and applied. This is a visible UX break in demos and undermines trust in “credits roll works.”

---

## Priority 1 (strongly recommended before publishing)

### 6) Make timeline sampling/render time deterministic by construction (avoid float accumulation drift)
Even if preview uses a fixed dt, you should avoid “accumulate `t += dt` forever” in render/export paths. The deterministic pattern is: compute frame index `i` and derive time `t = start + i / fps`. This removes drift, makes frame counts predictable, and keeps audio-event alignment easier to reason about. In preview mode, fixed-dt stepping is fine per your decisions; just ensure your render/export path is frame-index based.

### 7) Text layout should respect transform scale on both axes (or explicitly constrain it)
The current text layout logic measures width and wraps based on unscaled max width while only font size is scaled (often using `scale.y`). This leads to incorrect wrap/underline when `scale.x != 1`. Alpha choices that preserve sanity:
- Either incorporate `scale.x` into width calculation and max-width comparisons (recommended),
- Or document that non-uniform text scaling is unsupported in alpha and validate/error when `scale.x != scale.y`.
The “incorporate scale.x” approach is better for long-term ergonomics because users will naturally animate text scale.

### 8) Audio device ownership rules should be explicit and safe
If dropping an `AudioEngine` unconditionally closes the global audio device, you can accidentally break audio in multi-instance or future integration scenarios (e.g., preview + other audio systems). Alpha options:
- Document “single audio engine” as a constraint and enforce it (fail-fast on second init),
- Or implement a simple reference-counted/ownership guard so the device is closed only by the owner.
Even a documented constraint is acceptable for alpha if the failure mode is a clean `Result`, not a silent shutdown.

### 9) FFMPEG/FFPROBE preflight checks and dependency ergonomics
Since the project intentionally depends on external ffmpeg tooling, the library should:
- check for `ffmpeg` availability early (and `ffprobe` when video clips are used),
- return actionable errors (install instructions + override path hints),
- support override paths via config/CLI for locked-down environments.
This prevents “it crashed at the end of a render after 5 minutes” experiences and is essential for crates.io usability.

### 10) Clarify and document coordinate conventions and anchor behavior (especially text)
Your long-term plan includes configurable anchors (center/corners/custom). If alpha is “center origin” for shapes/images but text behaves like top-left, that is a paper cut that will cause confusing animation results. For alpha, either:
- implement a minimal `Anchor` model for all renderables (preferred), or
- explicitly document text anchor semantics and provide helper functions for alignment.
This is more about future-proofing than correctness, but it will pay dividends quickly.

---

## Priority 2 (quality bar upgrades: tests, docs, examples)

### 11) Add a small, high-value unit test suite for pure logic (and aim for high coverage)
Full coverage is a good goal, but the highest ROI is testing logic that doesn’t require raylib/ffmpeg:
- keyframe track sampling and easing behavior,
- overlap resolution (“newest stomps”) for video segments,
- clip validation rules,
- event time shifting/cropping for audio windows,
- ffmpeg argument construction (string snapshots / golden tests).
This suite is what prevents regressions when you iterate on features post-alpha.

### 12) Improve code comments in “conceptual choke points”
You explicitly want more comments. Focus on places where the *why* matters:
- how time is sampled deterministically,
- why overlap resolution is implemented the way it is,
- the contract between timeline time and ffmpeg trimming,
- the markdown subset constraints and parsing strategy.
Avoid commenting obvious Rust syntax; prioritize “future you / future agent” understanding.

### 13) Examples should cover the alpha story end-to-end
Crates succeed when a user can run a demo in minutes. Minimum example set:
- “Hello timeline” (shapes + image, layering),
- “Render MP4 with audio” (background + bounce),
- “Credits roll” (markdown subset + scroll),
- “Video stitching” (two clips + overlap stomps + trim).
Where assets are needed, prefer either:
- tiny permissively licensed assets included in-repo, or
- instructions to download sample assets separately so your crate stays lightweight and legally clean.

### 14) README + docs polish for external users
At minimum, README should include:
- what the tool is (“scriptable video compiler” in Rust),
- quickstart example (preview + render commands),
- dependency section (ffmpeg/ffprobe required; raylib build prerequisites),
- your “alpha constraints” list (stitching-only video, markdown subset, etc.),
- troubleshooting (ffmpeg missing, codec issues, mac/linux permissions).
This is also where you can document the decision you wrote for future_design_doc about ffmpeg packaging and dependencies.

---

## Priority 3 (nice-to-have, but improves long-term maintainability)

### 15) Replace `anyhow` in the public library API with structured errors
For crates.io libraries, typed errors improve ergonomics for downstream users. Consider:
- `thiserror` for `RenderError`, `AssetError`, `TimelineError`,
- keep `anyhow` only in binaries/examples if you add them.
This makes the library feel “Rust-native” and easier to integrate.

### 16) Feature flags to keep the core lightweight
As the project grows, feature flags reduce build friction:
- `preview-raylib`
- `render-ffmpeg`
- `video-clips`
- `text`
This also makes CI faster and improves “import as a library” adoption.

### 17) Archive or label prototype code to avoid contributor confusion
If `prototype/` is stale and uses old paths/deps, it becomes a trap for contributors and agents. Either:
- move it to `archive/` with a “legacy” README,
- or remove it from the published crate package (via `exclude` / `include` in Cargo.toml).
This is a small change that reduces onboarding friction.

### 18) Circle scaling semantics (document or extend)
If circle scaling uses only `scale.x`, non-uniform scaling is surprising. Alpha options:
- define circles as uniform scale (use min/max of x/y; document),
- or add ellipse support explicitly.
Not a blocker, but easy to tidy.

---

## Crates.io release hygiene (final steps)

### 19) Assets + licensing audit (required for a clean public release)
Before publishing:
- ensure any bundled audio/video/fonts are redistributable (and include their licenses),
- or exclude them from the crate and keep them only in repo examples.
This is a common reason early crates get pulled or criticized.

### 20) Cargo metadata and packaging configuration
Ensure Cargo.toml includes:
- `license`, `repository`, `readme`, `description`, `keywords`, `categories`
- sensible `include`/`exclude` so you don’t publish huge intermediates or stale prototypes
- bump version, tag, and confirm `cargo publish --dry-run` passes

### 21) CI expectations for alpha release
Consider adding:
- `cargo fmt --check`, `cargo clippy` (with reasonable lints), and tests
- a “release” workflow that builds examples (optional)
This is less about perfection and more about preventing accidental breakage right before a release.

---

## Notes from the additional agent review (tracked here explicitly)

These items are already integrated above, but listed here as a quick map of what came from the other review:
- Per-event SFX paths ignored (High) → addressed in Priority 0.1
- Video trim accuracy with `-ss` before input + `-c copy` (High) → addressed in Priority 0.2
- Text layout ignores horizontal scale (Medium) → addressed in Priority 1.7
- `ResourceCache::get_texture` panics (Medium) → addressed in Priority 0.3
- Audio engine closes global device on drop (Medium) → addressed in Priority 1.8
- No automated tests (Medium) → addressed in Priority 2.11
- Preview fixed dt “drift” concern (Low) → not treated as an issue because fixed dt is a deliberate product decision; document behavior and keep render deterministic
- Circle scaling ignores scale.y (Low) → addressed in Priority 3.18
- Prototype staleness (Low) → addressed in Priority 3.17
- Pre-alpha features requested: comments, tests, examples → addressed in Priority 2

---
