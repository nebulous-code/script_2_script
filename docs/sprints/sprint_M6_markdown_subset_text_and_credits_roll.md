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
- Text layout:
  - wraps within a bounding width
  - preserves newlines
- Rendering:
  - multiple styled “runs” drawn sequentially
- Credits roll:
  - text block moves upward over time using transform track

### Out-of-scope
- Full markdown spec (lists, code blocks, links)
- Advanced text shaping / kerning / font fallback
- Rich layout (columns, tables)

---

## Tasks (Agent Checklist)
- [ ] Define a `StyledText` object:
  - [ ] input: raw markdown string
  - [ ] output: `Vec<TextRun { text, style_flags }>`
- [ ] Implement parser for subset:
  - [ ] nested markers are optional; define behavior (no nesting allowed is fine)
- [ ] Implement text layout:
  - [ ] break into lines respecting `\n`
  - [ ] wrap lines by max width (approx measurement is acceptable for alpha)
- [ ] Render runs:
  - [ ] draw runs in order with correct font style
  - [ ] underline can be simulated by drawing a line under the run
- [ ] Credits roll example:
  - [ ] load `credits.md`
  - [ ] scroll upward from bottom to top over N seconds

---

## Deliverables
- `examples/m6_credits_roll.rs` that displays a scrolling credit list with bold/italic/underline.

---

## Acceptance Criteria
- [ ] Bold/italic/underline visibly differ.
- [ ] Newlines are preserved, wrapping works.
- [ ] Credits scroll smoothly and deterministically (based on time `t`).
- [ ] Works in preview and render output.

---

## Risks / Notes
- Font styling support depends on your rendering backend; you may need separate font files for bold/italic.
- Text width measurement can be approximate at first; document limitations.

---

## Open Decisions
- How to supply fonts (one font file vs separate bold/italic variants)?
- Do we allow nested formatting like `**bold *italic***` in alpha?
- Should wrapping be greedy word-wrap only, or allow character wrap for long tokens?
