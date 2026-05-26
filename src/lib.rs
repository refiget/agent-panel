pub mod activity;
pub mod adapter;
pub mod app;
pub mod cli;
pub mod clipboard;
pub mod desktop_notification;
pub mod event;
pub mod git;
pub mod group;
pub mod port;
pub(crate) mod process;
pub mod session;
pub mod state;
pub mod time;
pub mod tmux;
pub mod tool_name;
pub mod ui;
pub mod version;
pub mod worktree;

pub const SPINNER_ICON: &str = "●";
pub const SPINNER_PULSE: &[u8] = &[82, 78, 114, 150, 186, 150, 114, 78];
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub const RUNNING_GLYPHS: [&str; 10] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
pub const WAITING_GLYPHS: [&str; 4] = ["◐", "◓", "◑", "◒"];
pub const WAITING_PULSE: [u8; 4] = [178, 214, 172, 208];
pub const BG_PULSE: [u8; 2] = [25, 33];
pub const ATTN_PULSE: [u8; 2] = [208, 214];
