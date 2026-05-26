use ratatui::{
    style::Style,
    text::{Line, Span},
};

use super::ctx::RowCtx;
use crate::tmux::PaneStatus;
use crate::ui::text::{display_width, truncate_to_width, wait_reason_label};

pub(super) fn task_progress_row(
    task_progress: Option<&crate::activity::TaskProgress>,
    ctx: &RowCtx,
) -> Option<Line<'static>> {
    use crate::activity::TaskStatus;
    let progress = task_progress?;
    if progress.is_empty() {
        return None;
    }

    let completed = progress.completed_count();
    let total = progress.total();
    let task_color = ctx.theme.task_progress;
    let dim = ratatui::style::Color::Indexed(238);

    let mut filled = String::new();
    let mut unfilled = String::new();
    for (_, status) in &progress.tasks {
        match status {
            TaskStatus::Completed => filled.push_str("✔"),
            TaskStatus::InProgress => filled.push_str("◼"),
            TaskStatus::Pending => unfilled.push_str("▒"),
        }
    }

    let count = format!("  {}/{}", completed, total);
    let count_dw = display_width(&count);
    let filled_dw = display_width(&filled);
    let unfilled_dw = display_width(&unfilled);
    let total_dw = 2 + filled_dw + unfilled_dw + count_dw;

    Some(ctx.row_line(
        vec![
            Span::styled("  ", ctx.apply_bg(Style::default())),
            Span::styled(filled, ctx.apply_bg(Style::default().fg(task_color))),
            Span::styled(unfilled, ctx.apply_bg(Style::default().fg(dim))),
            Span::styled(count, ctx.apply_bg(Style::default().fg(dim))),
        ],
        total_dw,
    ))
}

pub(super) fn subagent_rows(subagents: &[String], ctx: &RowCtx) -> Vec<Line<'static>> {
    if subagents.is_empty() {
        return Vec::new();
    }
    let theme = ctx.theme;
    let subagent_color = theme.subagent;
    let tree_color = theme.text_muted;
    let last_idx = subagents.len() - 1;
    let mut out = Vec::with_capacity(subagents.len());
    for (i, sa) in subagents.iter().enumerate() {
        let connector = if i == last_idx { "└ " } else { "├ " };
        let numbered = if sa.contains('#') {
            sa.clone()
        } else {
            format!("{} #{}", sa, i + 1)
        };
        let prefix = format!("  {}", connector);
        let prefix_dw = display_width(&prefix);
        let max_sa_w = ctx.inner_width.saturating_sub(prefix_dw);
        let truncated_sa = truncate_to_width(&numbered, max_sa_w);
        let text_dw = prefix_dw + display_width(&truncated_sa);
        out.push(ctx.row_line(
            vec![
                Span::styled(prefix, ctx.apply_bg(Style::default().fg(tree_color))),
                Span::styled(
                    truncated_sa,
                    ctx.apply_bg(Style::default().fg(subagent_color)),
                ),
            ],
            text_dw,
        ));
    }
    out
}

pub(super) fn wait_reason_row(
    wait_reason: &str,
    status: &PaneStatus,
    ctx: &RowCtx,
) -> Option<Line<'static>> {
    if wait_reason.is_empty() {
        return None;
    }
    let reason = wait_reason_label(wait_reason);
    let text = format!("  ◈ {}", reason);
    let text_dw = display_width(&text);
    let amber_bg = ratatui::style::Color::Indexed(52);
    let reason_color = if matches!(status, PaneStatus::Error) {
        ctx.theme.status_error
    } else {
        ctx.theme.wait_reason
    };
    // Build spans with amber background directly — bypass ctx.apply_bg so the
    // amber bg always shows regardless of selection state.
    let mut spans = Vec::with_capacity(4);
    spans.push(Span::styled(ctx.marker_char, ctx.marker_style));
    spans.push(Span::styled(" ", Style::default().bg(amber_bg)));
    spans.push(Span::styled(
        text,
        Style::default().fg(reason_color).bg(amber_bg),
    ));
    let padding_len = ctx.inner_width.saturating_sub(text_dw);
    if padding_len > 0 {
        spans.push(Span::styled(
            " ".repeat(padding_len),
            Style::default().bg(amber_bg),
        ));
    }
    Some(Line::from(spans))
}

pub(super) fn background_hint_row(ctx: &RowCtx, cmd: &str) -> Line<'static> {
    const PREFIX: &str = "  $ ";
    let room = ctx.inner_width.saturating_sub(display_width(PREFIX));
    let shown = truncate_to_width(cmd.trim(), room);
    let text = format!("{PREFIX}{shown}");
    let text_dw = display_width(&text);
    ctx.row_line(
        vec![Span::styled(
            text,
            ctx.apply_bg(Style::default().fg(ctx.theme.status_running)),
        )],
        text_dw,
    )
}

pub(super) fn prompt_rows(pane: &crate::tmux::PaneInfo, ctx: &RowCtx) -> Vec<Line<'static>> {
    let theme = ctx.theme;
    let is_response = pane.prompt_is_response;
    let prompt_color = if ctx.active {
        theme.text_active
    } else {
        theme.text_inactive
    };

    if is_response {
        let bg = ratatui::style::Color::Indexed(22);
        let fg = ratatui::style::Color::Indexed(238);
        let arrow_color = ratatui::style::Color::Indexed(71);
        let prefix = "← ";
        let prefix_dw = display_width(prefix);
        let budget = ctx.inner_width.saturating_sub(2 + prefix_dw);
        let truncated = truncate_to_width(&pane.prompt, budget);
        let text_dw = prefix_dw + display_width(&truncated);
        return vec![ctx.row_line(
            vec![
                Span::styled(prefix.to_string(), Style::default().fg(arrow_color).bg(bg)),
                Span::styled(truncated, Style::default().fg(fg).bg(bg)),
            ],
            text_dw,
        )];
    }

    let indent = "  ";
    let budget = ctx.inner_width.saturating_sub(display_width(indent));
    let truncated = truncate_to_width(&pane.prompt, budget);
    let text = format!("{}{}", indent, truncated);
    let text_dw = display_width(&text);
    vec![ctx.row_line(
        vec![Span::styled(
            text,
            ctx.apply_bg(Style::default().fg(prompt_color)),
        )],
        text_dw,
    )]
}

pub(super) fn idle_hint_row(ctx: &RowCtx) -> Line<'static> {
    let text = "  Waiting for prompt…";
    let text_dw = display_width(text);
    let idle_color = if ctx.active {
        ctx.theme.text_active
    } else {
        ctx.theme.text_inactive
    };
    ctx.row_line(
        vec![Span::styled(
            text.to_string(),
            ctx.apply_bg(Style::default().fg(idle_color)),
        )],
        text_dw,
    )
}
