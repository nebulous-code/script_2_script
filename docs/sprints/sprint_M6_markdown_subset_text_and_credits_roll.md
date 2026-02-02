# Sprint Goal (M6)
Add text rendering with a markdown subset and credits-roll capability:
- bold, italic, underline
- wrapping + respects newline characters
- a credits roll effect (move text upward over time)

---

## Scope

### In-scope
- Markdown subset parser:
  - `**bold**`
  - `*italic*`
  - `__underline__`
  - newline support
  - **nested formatting supported** (e.g., `**bold *italic***`) for this subset
- Fonts:
  - a `FontFamily` concept with optional variants (regular/bold/italic/bold-italic)
  - deterministic fallback behavior when variants are missing
- Text layout:
  - wraps within a bounding width
  - preserves newlines
  - left-justified only (no justification stretching)
- Rendering:
  - multiple styled “runs” drawn sequentially
- Credits roll:
  - text block moves upward over time using transform track

### Out-of-scope
- Full markdown spec (lists, code blocks, links)
- Advanced text shaping / kerning / font fallback across multiple font families
- Rich layout (columns, tables)
- Hyphenation / dictionary-based word breaking

---

## Tasks (Agent Checklist)

### Font support
- [x] Define a `FontFamily` object (or similar):
  - [x] `regular` font is required
  - [x] `bold`, `italic`, `bold_italic` are optional
  - [x] Provide a method like `resolve(style_flags) -> &Font` with fallback rules:
    - [x] if requested face exists, use it
    - [x] otherwise fallback to `regular`
  - [x] Document fallback behavior (missing bold/italic renders as regular)

### Styled text parsing
- [x] Define a `StyledText` object:
  - [x] input: raw markdown string
  - [x] output: `Vec<TextRun { text, style_flags }>`
- [x] Implement parser for subset with nesting:
  - [x] supported markers: `**` (bold), `*` (italic), `__` (underline)
  - [x] use a simple style-stack approach
  - [x] best-effort parsing:
    - [x] malformed/unbalanced markers do not hard-error
    - [x] treat unknown/invalid sequences as literal text (or degrade gracefully)

### Text layout
- [x] Implement line breaking:
  - [x] split on `\n` (preserve explicit newlines)
  - [x] greedy word-wrap by whitespace within a bounding max width
  - [x] **fallback behavior for long tokens**:
    - [x] if a single token exceeds the max width, allow character wrap for that token only
    - [x] no hyphen insertion required for alpha
- [x] Width measurement:
  - [x] approximate measurement is acceptable for alpha, but must be deterministic
  - [x] document limitations

### Rendering
- [x] Render runs:
  - [x] draw runs in order with correct font face from `FontFamily`
  - [x] underline simulated by drawing a line under the run (per-run)
  - [x] ensure opacity and transforms apply consistently
- [x] Verify compatibility:
  - [x] works in preview (raylib window)
  - [x] works in render output (ffmpeg pipeline)

### Credits roll example
- [x] Add `examples/m6_credits_roll.rs`:
  - Note: Demo code should include clear, beginner-friendly comments explaining what each section does.
  - [x] load `credits.md`
  - [x] parse styled runs
  - [x] layout into lines within a bounding width
  - [x] scroll upward from bottom to top over N seconds using the existing animation system

---

## Deliverables
- `examples/m6_credits_roll.rs` that displays a scrolling credit list with bold/italic/underline.

---

## Acceptance Criteria
- [x] Bold/italic/underline visibly differ (when font variants exist; otherwise documented fallback).
- [x] Nested formatting works for the supported subset.
- [x] Newlines are preserved; wrapping works and is deterministic.
- [x] Long tokens do not overflow the bounding box (character-wrap fallback).
- [x] Credits scroll smoothly and deterministically (based on time `t`).
- [x] Output looks consistent in preview and in rendered MP4 output.

---

## Risks / Notes
- Font styling depends on separate font files in most pipelines; alpha uses `FontFamily` with explicit optional variants.
- Text width measurement can be approximate; document limitations.
- Full markdown and rich typography remain post-alpha.

---

## Open Decisions
- None for M6 (font family + fallback, nested formatting, and wrap behavior are now locked in).
