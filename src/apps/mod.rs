pub mod app_windows;
mod scoreboard_app;
pub use scoreboard_app::ScoreBoardApp;
mod challenge_info;
pub use challenge_info::ChallengeInfoApp;
pub mod login_app;
pub use login_app::LoginApp;
pub mod binary_upload;
pub mod code_editor;
mod password_reset_app;
use crate::helpers::AppState;
pub use password_reset_app::PasswordResetApp;
use std::sync::{Arc, Mutex};

/// Something to view in the demo windows
pub trait View {
    fn ui(&mut self, ui: &mut egui::Ui);
}

/// Something to view
pub trait App {
    /// `&'static` so we can also use it as a key to store open/close state.
    fn name(&self) -> &'static str;

    /// Show windows, etc
    fn show(&mut self, ctx: &egui::Context, open: &mut bool);

    fn set_app_state_ref(&mut self, app_state: Arc<Mutex<AppState>>);
}
