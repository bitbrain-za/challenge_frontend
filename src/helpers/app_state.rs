#[derive(Clone)]
pub enum LoginState {
    LoggedIn(String),
    LoggedOut,
}

pub struct AppState {
    pub counter: usize,
    pub logged_in: LoginState,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            counter: 1,
            logged_in: LoginState::LoggedOut,
        }
    }
}
