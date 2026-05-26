use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
};

use super::ctx::RowCtx;
use crate::tmux::PaneStatus;
use crate::ui::text::{display_width, elapsed_label, truncate_to_width};
use crate::{ATTN_PULSE, BG_PULSE, RUNNING_GLYPHS, SPINNER_PULSE, WAITING_GLYPHS, WAITING_PULSE};

pub(super) fn status_row(
    pane: &crate::tmux::PaneInfo,
    ctx: &RowCtx,
    spinner_frame: usize,
    now: u64,
) -> Line<'static> {
    use crate::tmux::PermissionMode;
    let theme = ctx.theme;

    let (icon, icon_color) = animated_icon(&pane.status, pane.attention, spinner_frame);
    let title_raw: &str = if pane.session_name.is_empty() {
        pane.agent.label()
    } else {
        &pane.session_name
    };
    let badge = pane.permission_mode.badge();
    let elapsed = elapsed_label(pane.started_at, now);

    let title_fg = theme.agent_color(&pane.agent);
    let elapsed_fg = if pane.status.is_active() {
        theme.text_active
    } else {
        theme.text_muted
    };

    let badge_extra = if badge.is_empty() { 0 } else { 1 };
    let fixed_width = display_width(icon) + 1 + badge_extra + display_width(badge);
    let elapsed_width = display_width(&elapsed);
    let elapsed_gap = usize::from(elapsed_width > 0);
    let title_budget = ctx
        .inner_width
        .saturating_sub(fixed_width + elapsed_gap + elapsed_width);
    let title = truncate_to_width(title_raw, title_budget);

    let left_width = fixed_width + display_width(&title);
    let available_for_elapsed = ctx.inner_width.saturating_sub(left_width);
    let elapsed = truncate_to_width(&elapsed, available_for_elapsed);
    let elapsed_width = display_width(&elapsed);

    let mut left_spans: Vec<Span<'static>> = Vec::with_capacity(3);
    left_spans.push(Span::styled(
        icon.to_string(),
        ctx.apply_bg(Style::default().fg(icon_color)),
    ));
    left_spans.push(Span::styled(
        format!(" {}", title),
        ctx.apply_bg(Style::default().fg(title_fg)),
    ));
    if !badge.is_empty() {
        let badge_color = match pane.permission_mode {
            PermissionMode::BypassPermissions => theme.badge_danger,
            PermissionMode::Auto => theme.badge_auto,
            PermissionMode::DontAsk => theme.badge_auto,
            PermissionMode::Plan => theme.badge_plan,
            PermissionMode::AcceptEdits => theme.badge_auto,
            PermissionMode::Defer => theme.badge_auto,
            PermissionMode::Default => theme.text_muted,
        };
        left_spans.push(Span::styled(
            format!(" {}", badge),
            ctx.apply_bg(Style::default().fg(badge_color)),
        ));
    }

    let right_spans = vec![Span::styled(
        elapsed,
        ctx.apply_bg(Style::default().fg(elapsed_fg)),
    )];

    ctx.row_line_split(left_spans, left_width, right_spans, elapsed_width)
}

pub(super) fn animated_icon(
    status: &PaneStatus,
    attention: bool,
    frame: usize,
) -> (&'static str, Color) {
    if attention {
        return ("◉", Color::Indexed(ATTN_PULSE[frame % 2]));
    }
    match status {
        PaneStatus::Running => (
            RUNNING_GLYPHS[frame % RUNNING_GLYPHS.len()],
            Color::Indexed(SPINNER_PULSE[frame % SPINNER_PULSE.len()]),
        ),
        PaneStatus::Background => ("⊙", Color::Indexed(BG_PULSE[frame % 2])),
        PaneStatus::Waiting => (
            WAITING_GLYPHS[(frame / 2) % WAITING_GLYPHS.len()],
            Color::Indexed(WAITING_PULSE[(frame / 2) % WAITING_PULSE.len()]),
        ),
        PaneStatus::Idle => ("○", Color::Indexed(236)),
        PaneStatus::Error => ("⊗", Color::Indexed(203)),
        PaneStatus::Unknown => ("·", Color::Indexed(235)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn animated_icon_attention_overrides_any_status() {
        let (glyph, _) = animated_icon(&PaneStatus::Idle, true, 0);
        assert_eq!(glyph, "◉");
        let (glyph, _) = animated_icon(&PaneStatus::Running, true, 0);
        assert_eq!(glyph, "◉");
    }

    #[test]
    fn animated_icon_running_cycles_braille_glyphs() {
        let (g0, _) = animated_icon(&PaneStatus::Running, false, 0);
        let (g1, _) = animated_icon(&PaneStatus::Running, false, 1);
        assert_eq!(g0, "⠋");
        assert_eq!(g1, "⠙");
    }

    #[test]
    fn animated_icon_waiting_uses_half_speed() {
        let (g0, _) = animated_icon(&PaneStatus::Waiting, false, 0);
        let (g1, _) = animated_icon(&PaneStatus::Waiting, false, 1);
        assert_eq!(g0, g1, "waiting advances every 2 frames");
        let (g2, _) = animated_icon(&PaneStatus::Waiting, false, 2);
        assert_ne!(g0, g2, "waiting must advance at frame 2");
    }

    #[test]
    fn animated_icon_static_statuses() {
        let (idle, _) = animated_icon(&PaneStatus::Idle, false, 0);
        let (err, _) = animated_icon(&PaneStatus::Error, false, 0);
        let (unk, _) = animated_icon(&PaneStatus::Unknown, false, 0);
        assert_eq!(idle, "○");
        assert_eq!(err, "⊗");
        assert_eq!(unk, "·");
    }

    #[test]
    fn animated_icon_background_uses_ring_glyph() {
        let (bg, _) = animated_icon(&PaneStatus::Background, false, 0);
        assert_eq!(bg, "⊙");
    }
}
