use crate::session;
use crate::state::AppState;
use crate::ui;

/// Construct and prime the initial [`AppState`] before the event loop starts.
///
/// Equivalent to the original `run_app` prelude in `src/main.rs`: installs the
/// color theme/icons from tmux options, loads global filter state, resolves
/// the Claude plugin install version once at startup, seeds session names
/// synchronously so `/rename` labels render on the first frame, and performs
/// the first refresh pass.
pub(super) fn init_state(tmux_pane: String) -> AppState {
    let mut state = AppState::new(tmux_pane);
    state.theme = ui::colors::ColorTheme::from_tmux();
    state.icons = ui::icons::StatusIcons::from_tmux();
    state.bottom_panel_height = 0;
    state.pet_enabled = ui::pet_enabled_from_tmux();
    state.global.load_from_tmux();
    state.refresh();

    super::render::refresh_git_for_focused_pane(&mut state);

    // Populate session names synchronously before the first draw so
    // `/rename`-assigned labels show up without waiting for the first
    // background scan tick.
    state.sessions.names = session::scan_session_names();
    state.sessions.dirty = true;
    state.refresh();

    state
}
