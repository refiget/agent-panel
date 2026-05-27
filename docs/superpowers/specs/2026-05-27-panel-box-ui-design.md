# Panel Box UI Design Spec

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Wrap the agents pane in a full rounded-corner panel box (matching the existing Git panel style) with lazygit-style focus highlighting — the entire border changes to accent color when focused, dims when not.

**Architecture:** Use ratatui's `Block` widget with `Borders::ALL` + `BorderType::Rounded` to own the outer frame. Inner dividers (`├──┤`) are drawn by overwriting specific buffer cells after block rendering. All content renders into the block's inner `Rect`, reducing available width by 2 columns.

**Tech Stack:** Rust, ratatui 0.30, existing `ColorTheme`, `PaneLayout`, `Focus` state

---

## Visual Target

```
╭──────────────────────────────────────────╮   ← accent when agents focused
│ [≡4] │ ●3 │ ⊙0 │ ◐1 │ ○0 │ ⊗0          │   filter row
├──────────────────────────────────────────┤   ← accent when focused
│                               ‹ — ›     │   repo row
├──────────────────────────────────────────┤   ← accent when focused
│ ── ~/.config/tmux ──────────────── [+]  │   group header
│ ▌ ● claude  acc           3m            │   selected pane (selection_bg)
│   ← 完成了前端重构任务…                   │   response row
│                                          │
│   ● codex   default       12m           │
│   ◈ 等待确认文件删除权限                  │
╰──────────────────────────────────────────╯   ← accent when focused

╭─ Git ── main  +42/-8 ────────────────────╮   ← accent when git focused
│ ● staged   src/ui/colors.rs  +15/-3      │
╰──────────────────────────────────────────╯
```

When agents panel is NOT focused, all border chars (`╭ ╮ ╰ ╯ │ ─ ├ ┤`) render in `border_inactive` color. When focused, all render in `accent` color.

---

## Layout Arithmetic

### Before (no outer box)
```
PaneLayout occupies: full agents Rect
  filter_area  = y+0, h=1
  sep1_area    = y+1, h=1  (rendered as plain ────)
  secondary    = y+2, h=1
  sep2_area    = y+3, h=1  (rendered as plain ────)
  list_area    = y+4..end

Row content width = area.width
inner_width (RowCtx) = area.width - 2   (marker + space)
```

### After (with outer box, Block border = 1 cell on each side)
```
Block outer Rect = agents Rect
Block inner Rect = { x: area.x+1, y: area.y+1, w: area.width-2, h: area.height-2 }

PaneLayout occupies: Block inner Rect
  filter_area  = inner.y+0, h=1
  sep1_area    = inner.y+1, h=1  (drawn as ├──┤ by box renderer, NOT by separator fn)
  secondary    = inner.y+2, h=1
  sep2_area    = inner.y+3, h=1  (drawn as ├──┤ by box renderer, NOT by separator fn)
  list_area    = inner.y+4..inner.y+inner.height-1
                              ^^ inner rect ends here; Block's ╰──╯ is at area.y+area.height-1, outside inner

Row content width = inner.width          (= area.width - 2)
inner_width (RowCtx) = inner.width - 2  (= area.width - 4)
```

The bottom border `╰──╯` is part of the Block — no change to list_area end calculation needed; Block draws it outside the inner rect.

---

## Inner Divider Rendering

After `block.render(area, buf)`, the sep1 and sep2 rows inside the inner rect have `│` drawn by the Block at `x = area.x` and `x = area.x + area.width - 1`. Overwrite these with `├` and `┤`, and fill the inner span with `─`:

```rust
fn draw_inner_divider(buf: &mut Buffer, area: Rect, inner: Rect, row_y: u16, style: Style) {
    // Left junction: overwrite Block's │ with ├
    if let Some(cell) = buf.cell_mut(Position { x: area.x, y: row_y }) {
        cell.set_char('├');
        cell.set_style(style);
    }
    // Fill inner span with ─
    for x in inner.x..(inner.x + inner.width) {
        if let Some(cell) = buf.cell_mut(Position { x, y: row_y }) {
            cell.set_char('─');
            cell.set_style(style);
        }
    }
    // Right junction: overwrite Block's │ with ┤
    if let Some(cell) = buf.cell_mut(Position { x: area.x + area.width - 1, y: row_y }) {
        cell.set_char('┤');
        cell.set_style(style);
    }
}
```

Call this for `layout.sep1_area.y` and `layout.sep2_area.y` after rendering the block.

---

## Focus State Wiring

The agents panel is focused when `state.focus_state.sidebar_focused && state.focus_state.focus != Focus::Bottom`.

```rust
fn agents_border_style(state: &AppState) -> Style {
    let focused = state.focus_state.sidebar_focused
        && state.focus_state.focus != Focus::Bottom;
    if focused {
        Style::default().fg(state.theme.accent)
    } else {
        Style::default().fg(state.theme.border_inactive)
    }
}
```

The Block is built with `block.border_style(border_style)`. The inner dividers use the same style.

The Git panel (`bottom.rs`) already implements this pattern — its border changes to `accent` when `state.focus_state.focus == Focus::Bottom`. No changes needed to `bottom.rs`.

---

## Click Target Adjustments

`repo_button_col` in `render_secondary_header` is currently computed relative to the full area width. After adding the box, `width` passed to `render_secondary_header` becomes `inner.width` (= `area.width - 2`). The returned column value is relative to `inner.x`.

The click handler in `panes.rs` receives mouse `column` in screen coords. Currently it compares against `repo_button_col` directly. After the change, the comparison must account for the block's left border offset:

```rust
// Before:
if col >= layout.repo_button_col { ... }

// After (inner.x = area.x + 1):
if col >= inner.x + layout.repo_button_col { ... }
```

Spawn button `[+]` and remove marker `×` positions stored in `CollectedRows.pending_spawn` and `pending_remove` are line-index based, not column based — no change needed for those.

`notices_button_col` is similarly offset by `inner.x`.

---

## Files to Change

| File | Change |
|------|--------|
| `src/ui/panes.rs` | Wrap agents area in Block; compute inner rect; call `draw_inner_divider` for sep rows; update click target offset logic; remove `render_separator_into` calls for sep1/sep2 (block does them via `draw_inner_divider`) |
| `src/ui/panes/filter_bar.rs` | Pass `inner.width` (not `area.width`) to `render_secondary_header` |
| `src/ui/panes/row_collector.rs` | Pass `inner.width` as `width` to `collect()` |
| `src/ui/panes/row.rs` | `inner_width` in `RowCtx` derives from `inner.width - 2` (unchanged logic, just smaller number) |
| `src/ui/mod.rs` | No change — layout split is unchanged |
| `src/ui/bottom.rs` | No change — already has correct box + focus behavior |

---

## Separator Rendering

`render_separator_into` currently renders plain `─` lines for sep1 and sep2. After the block, those rows are drawn by `draw_inner_divider` instead. The `render_separator_into` function is no longer called for sep1/sep2 — the content at those rows is irrelevant because `draw_inner_divider` overwrites the entire row. The function can be removed if no other callers exist.

---

## Tests to Update

- `PaneLayout::compute` tests: update expected `list_area` y-offset (stays at inner.y+4, but inner.y itself shifts by 1 when computing from a non-zero-origin area)
- `render_secondary_header` width tests: the `width` argument is now `inner.width = area.width - 2`, so expected column positions shift accordingly
- Any test that checks screen-coordinate click targets needs the `+1` offset for the block border

---

## Out of Scope

- Pet scene borders — no change, it remains a decorative separator
- Popup modals — no change to popup positioning logic
- Color palette — border colors use existing `accent` and `border_inactive` theme values
- Git panel — already correct, no changes
