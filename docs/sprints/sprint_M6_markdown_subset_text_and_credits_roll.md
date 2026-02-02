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
- [ ] Define a `FontFamily` object (or similar):
  - [ ] `regular` font is required
  - [ ] `bold`, `italic`, `bold_italic` are optional
  - [ ] Provide a method like `resolve(style_flags) -> &Font` with fallback rules:
    - [ ] if requested face exists, use it
    - [ ] otherwise fallback to `regular`
  - [ ] Document fallback behavior (missing bold/italic renders as regular)

### Styled text parsing
- [ ] Define a `StyledText` object:
  - [ ] input: raw markdown string
  - [ ] output: `Vec<TextRun { text, style_flags }>`
- [ ] Implement parser for subset with nesting:
  - [ ] supported markers: `**` (bold), `*` (italic), `__` (underline)
  - [ ] use a simple style-stack approach
  - [ ] best-effort parsing:
    - [ ] malformed/unbalanced markers do not hard-error
    - [ ] treat unknown/invalid sequences as literal text (or degrade gracefully)

### Text layout
- [ ] Implement line breaking:
  - [ ] split on `\n` (preserve explicit newlines)
  - [ ] greedy word-wrap by whitespace within a bounding max width
  - [ ] **fallback behavior for long tokens**:
    - [ ] if a single token exceeds the max width, allow character wrap for that token only
    - [ ] no hyphen insertion required for alpha
- [ ] Width measurement:
  - [ ] approximate measurement is acceptable for alpha, but must be deterministic
  - [ ] document limitations

### Rendering
- [ ] Render runs:
  - [ ] draw runs in order with correct font face from `FontFamily`
  - [ ] underline simulated by drawing a line under the run (per-run)
  - [ ] ensure opacity and transforms apply consistently
- [ ] Verify compatibility:
  - [ ] works in preview (raylib window)
  - [ ] works in render output (ffmpeg pipeline)

### Credits roll example
- [ ] Add `examples/m6_credits_roll.rs`:
  - [ ] load `credits.md`
  - [ ] parse styled runs
  - [ ] layout into lines within a bounding width
  - [ ] scroll upward from bottom to top over N seconds using the existing animation system

---

## Deliverables
- `examples/m6_credits_roll.rs` that displays a scrolling credit list with bold/italic/underline.

---

## Acceptance Criteria
- [ ] Bold/italic/underline visibly differ (when font variants exist; otherwise documented fallback).
- [ ] Nested formatting works for the supported subset.
- [ ] Newlines are preserved; wrapping works and is deterministic.
- [ ] Long tokens do not overflow the bounding box (character-wrap fallback).
- [ ] Credits scroll smoothly and deterministically (based on time `t`).
- [ ] Output looks consistent in preview and in rendered MP4 output.

---

## Risks / Notes
- Font styling depends on separate font files in most pipelines; alpha uses `FontFamily` with explicit optional variants.
- Text width measurement can be approximate; document limitations.
- Full markdown and rich typography remain post-alpha.

---

## Open Decisions
- None for M6 (font family + fallback, nested formatting, and wrap behavior are now locked in).
