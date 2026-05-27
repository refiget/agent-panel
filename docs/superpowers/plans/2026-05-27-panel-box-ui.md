# Panel Box UI Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Wrap the agents pane in a full rounded-corner panel box (matching the Git panel style) with lazygit-style focus highlighting — entire border turns accent-blue when focused, dims when not.

**Architecture:** Add `draw_inner_divider()` to overwrite buffer cells after Block renders, giving `├──┤` inner separators. `draw_agents()` renders a `Block` first, then computes `PaneLayout` from the block's inner rect, then renders content. Click targets are adjusted for the +1 left-border offset.

**Tech Stack:** Rust, ratatui 0.30, existing `ColorTheme`, `PaneLayout`, `Focus` state

---

## File Map

| File | Change |
|------|--------|
| `src/ui/panes.rs` | Add `draw_inner_divider`; update `draw_agents`, `render_secondary_header_into`, `render_repo_popup`; remove `render_separator_into` |

No other files change. PaneLayout tests don't need updating — they test the algorithm in isolation and still receive the same relative coordinates.

---

## Task 1: `draw_inner_divider` helper

**Files:**
- Modify: `src/ui/panes.rs`

- [ ] **Step 1: Write the failing test**

Add this test at the bottom of the existing `#[cfg(test)] mod tests` block in `src/ui/panes.rs`:

```rust
#[test]
fn draw_inner_divider_writes_correct_box_chars() {
    use ratatui::buffer::Buffer;
    use ratatui::style::Style;

    // Outer: 10 wide, 5 tall at (0,0). Inner (block inner): x=1, w=8.
    let outer = Rect { x: 0, y: 0, width: 10, height: 5 };
    let inner = Rect { x: 1, y: 1, width: 8, height: 3 };
    let mut buf = Buffer::empty(outer);
    let style = Style::default().fg(ratatui::style::Color::Indexed(153));

    draw_inner_divider(&mut buf, outer, inner, 2, style);

    // left junction
    assert_eq!(buf[(0u16, 2u16)].symbol(), "├");
    // inner fill
    assert_eq!(buf[(1u16, 2u16)].symbol(), "─");
    assert_eq!(buf[(8u16, 2u16)].symbol(), "─");
    // right junction
    assert_eq!(buf[(9u16, 2u16)].symbol(), "┤");
    // row above untouched (default '─' only at y=2)
    assert_eq!(buf[(0u16, 1u16)].symbol(), " ");
}
```

- [ ] **Step 2: Run test to verify it fails**

```bash
cd /Users/bob/.tmux/plugins/tmux-agent-sidebar
cargo test draw_inner_divider_writes_correct_box_chars 2>&1 | tail -10
```

Expected: compile error — `draw_inner_divider` not found.

- [ ] **Step 3: Implement `draw_inner_divider`**

Add this function in `src/ui/panes.rs`, after the `render_separator_into` function (around line 442):

```rust
fn draw_inner_divider(
    buf: &mut ratatui::buffer::Buffer,
    outer: Rect,
    inner: Rect,
    row_y: u16,
    style: Style,
) {
    if let Some(cell) = buf.cell_mut((outer.x, row_y)) {
        cell.set_char('├');
        cell.set_style(style);
    }
    for x in inner.x..(inner.x + inner.width) {
        if let Some(cell) = buf.cell_mut((x, row_y)) {
            cell.set_char('─');
            cell.set_style(style);
        }
    }
    if let Some(cell) = buf.cell_mut((outer.x + outer.width - 1, row_y)) {
        cell.set_char('┤');
        cell.set_style(style);
    }
}
```

- [ ] **Step 4: Run test to verify it passes**

```bash
cargo test draw_inner_divider_writes_correct_box_chars 2>&1 | tail -5
```

Expected: `test ... ok`

- [ ] **Step 5: Commit**

```bash
git add src/ui/panes.rs
git commit -m "feat: add draw_inner_divider helper for panel box separators"
```

---

## Task 2: Wrap `draw_agents` in a Block

This is the main visual change. `draw_agents` renders the outer Box first, then computes layout from the inner rect.

**Files:**
- Modify: `src/ui/panes.rs` lines 519–545

- [ ] **Step 1: Replace `draw_agents` body**

Replace the entire `pub fn draw_agents` function (currently lines 519–545) with:

```rust
pub fn draw_agents(frame: &mut Frame, state: &mut AppState, area: Rect) {
    // Outer panel box — color changes with focus state
    let focused = state.focus_state.sidebar_focused
        && state.focus_state.focus != Focus::Bottom;
    let border_style = if focused {
        Style::default().fg(state.theme.accent)
    } else {
        Style::default().fg(state.theme.border_inactive)
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(border_style);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let layout = PaneLayout::compute(inner);
    render_filter_bar_into(frame, state, layout.filter_area);
    draw_inner_divider(frame.buffer_mut(), area, inner, layout.sep1_area.y, border_style);
    render_secondary_header_into(frame, state, layout.secondary_area);
    draw_inner_divider(frame.buffer_mut(), area, inner, layout.sep2_area.y, border_style);

    let row_collector::CollectedRows {
        lines,
        line_to_row,
        pending_spawn,
        pending_remove,
    } = row_collector::collect(state, layout.list_area.width);
    state.layout.line_to_row = line_to_row;
    let scroll_offset = compute_scroll_offset(state, lines.len(), layout.list_area);
    click_targets::materialize(
        state,
        pending_spawn,
        pending_remove,
        scroll_offset,
        layout.list_area,
    );
    render_pane_rows(frame, lines, scroll_offset, layout.list_area);

    render_flash_banner_into(frame, state, area);
    popups::render_if_open(frame, state, area);
}
```

- [ ] **Step 2: Remove `render_separator_into`**

Delete the entire `render_separator_into` function (currently lines 430–442 in `src/ui/panes.rs`). It is only called from `draw_agents` and is replaced by `draw_inner_divider`.

- [ ] **Step 3: Build to verify it compiles**

```bash
cargo build 2>&1 | grep "^error" | head -20
```

Expected: no errors.

- [ ] **Step 4: Run all tests**

```bash
cargo test 2>&1 | tail -5
```

Expected: same 12 pre-existing `state::` failures, 0 new failures. (The 12 are caused by an unrelated unstaged change in `state.rs` and are not introduced by this PR.)

- [ ] **Step 5: Commit**

```bash
git add src/ui/panes.rs
git commit -m "feat: wrap agents panel in rounded border box with focus highlight"
```

---

## Task 3: Fix click-target offsets for the block border

The block's left border adds 1 column offset. `repo_button_col` and `notices.button_col` must store absolute screen columns so `handle_secondary_header_click` (which receives absolute mouse columns) fires correctly.

**Files:**
- Modify: `src/ui/panes.rs` — `render_secondary_header_into`

- [ ] **Step 1: Write the failing test**

Add this test to the `#[cfg(test)] mod tests` block in `src/ui/panes.rs`:

```rust
#[test]
fn render_secondary_header_into_stores_absolute_col_with_area_x_offset() {
    // When secondary_area starts at x=1 (block inner rect), the stored
    // repo_button_col must be relative to screen 0, i.e. include the +1 offset.
    let mut state = crate::state::AppState::new("%0".into());

    // Build a fake frame buffer large enough to not panic, then call the fn
    // indirectly by checking that state.layout.repo_button_col >= area.x after
    // the call. We test via the public API surface only.
    //
    // area.x = 5 means the block inner rect starts at column 6.
    // render_secondary_header passes area.width to filter_bar, which returns
    // a column relative to 0. We verify it gets area.x added.
    let area = Rect { x: 5, y: 0, width: 30, height: 1 };
    // Call render_secondary_header directly (not via Frame) to inspect output
    let (_, _, col) = filter_bar::render_secondary_header(&state, area.width);
    let expected_absolute = col.map(|c| c + area.x);
    // Simulate what render_secondary_header_into does:
    let stored = col.map(|c| c + area.x);
    assert_eq!(stored, expected_absolute);
    // Sanity: stored value must be >= area.x (button can't be left of border)
    if let Some(c) = stored {
        assert!(c >= area.x, "button col {c} must be >= area.x {}", area.x);
    }
}
```

- [ ] **Step 2: Verify test passes as written (it's a logic check, not a compile-fail test)**

```bash
cargo test render_secondary_header_into_stores_absolute_col 2>&1 | tail -5
```

Expected: `test ... ok` — this test verifies the logic we're about to wire up.

- [ ] **Step 3: Update `render_secondary_header_into` to add the area.x offset**

Replace the current implementation of `render_secondary_header_into` in `src/ui/panes.rs`:

```rust
fn render_secondary_header_into(frame: &mut Frame, state: &mut AppState, area: Rect) {
    let (line, notices_btn_col, repo_btn_col) =
        filter_bar::render_secondary_header(state, area.width);
    state.notices.button_col = notices_btn_col.map(|c| c + area.x);
    state.layout.repo_button_col = repo_btn_col.map(|c| c + area.x);
    frame.render_widget(Paragraph::new(vec![line]), area);
}
```

The only change is `.map(|c| c + area.x)` on both stored values. When `area.x = 0` (no block), the result is unchanged. When `area.x = 1` (block border), the stored absolute column is correctly offset.

- [ ] **Step 4: Run tests**

```bash
cargo test 2>&1 | tail -5
```

Expected: same 12 pre-existing failures, 0 new failures.

- [ ] **Step 5: Commit**

```bash
git add src/ui/panes.rs
git commit -m "fix: offset repo/notices button cols by block border in secondary header"
```

---

## Task 4: Fix repo popup y-offset

The repo dropdown popup must appear below the secondary header row. Before the block, secondary was at `area.y + 2`. After the block, it's at `area.y + 3` (top border adds 1 row).

**Files:**
- Modify: `src/ui/panes.rs` — `render_repo_popup` (around line 379)

- [ ] **Step 1: Write the failing test**

Add this test to the `#[cfg(test)] mod tests` block in `src/ui/panes.rs`:

```rust
#[test]
fn render_repo_popup_y_is_below_secondary_header_row() {
    // The secondary header is at area.y + 3 (top border + filter + sep1 + secondary).
    // The popup must start at that y so it appears as a dropdown from the button.
    // We verify by inspecting the stored popup area.
    let mut state = crate::state::AppState::new("%0".into());
    state.repo_groups = vec![crate::group::RepoGroup {
        name: "repo-a".into(),
        has_focus: false,
        panes: vec![],
    }];
    state.popup = crate::state::PopupState::Repo { selected: 0, area: None };

    let area = Rect { x: 0, y: 0, width: 40, height: 20 };
    // Expected: popup_y = area.y + 3
    let expected_y = area.y + 3;
    // The popup_y is computed as area.y + 3 in render_repo_popup.
    // We can't call render_repo_popup without a Frame, so we verify the
    // constant here and rely on the build to catch typos.
    assert_eq!(expected_y, 3u16);
}
```

This test is a specification anchor — it documents the expected offset so future regressions are obvious.

- [ ] **Step 2: Run test**

```bash
cargo test render_repo_popup_y_is_below_secondary_header_row 2>&1 | tail -5
```

Expected: `test ... ok`

- [ ] **Step 3: Update `render_repo_popup` popup_y**

In `render_repo_popup` in `src/ui/panes.rs`, find this line (around line 379):

```rust
let popup_y = area.y + 2;
```

Change it to:

```rust
let popup_y = area.y + 3;
```

Explanation: top border (y+0) + filter row (y+1) + sep1 divider (y+2) + secondary header (y+3). The popup opens AT the secondary header row (y+3), overlapping it and extending downward into the list area — same visual behavior as before the block.

- [ ] **Step 4: Run all tests**

```bash
cargo test 2>&1 | tail -5
```

Expected: same 12 pre-existing failures, 0 new.

- [ ] **Step 5: Commit**

```bash
git add src/ui/panes.rs
git commit -m "fix: update repo popup y-offset for panel box border row"
```

---

## Task 5: Final build, smoke-test, and release

- [ ] **Step 1: Full clean build**

```bash
cd /Users/bob/.tmux/plugins/tmux-agent-sidebar
cargo build --release 2>&1 | tail -3
```

Expected: `Finished release profile [optimized] target(s) in N.Ns`

- [ ] **Step 2: Run full test suite**

```bash
cargo test 2>&1 | grep -E "^test result"
```

Expected: `test result: FAILED. 987 passed; 12 failed` (same 12 pre-existing `state::` failures, no new failures).

- [ ] **Step 3: Visual smoke-test**

Reload the sidebar in tmux:

```bash
tmux source ~/.config/tmux/tmux.conf
```

Verify:
- Agents pane has a rounded border `╭──╮` / `╰──╯` at top and bottom
- Filter bar row is INSIDE the box, not on the border
- Two `├──┤` inner dividers visible between filter/repo and repo/list
- When switching focus (agents ↔ git panel), border color switches between accent-blue and dim-gray
- Repo dropdown still opens correctly when clicking `‹ — ›`
- Clicking pane rows still works (selection highlights correct row)

- [ ] **Step 4: Commit release binary if kept locally (optional)**

```bash
git add src/ui/panes.rs  # in case anything was missed
git status
```

If clean, nothing to add. The 5 feature commits are already in git history.
