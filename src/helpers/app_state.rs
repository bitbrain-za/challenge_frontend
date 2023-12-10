use crate::helpers::ChallengeCollection;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub enum LoginState {
    LoggedIn,
    LoggedOut,
}

pub struct AppState {
    pub counter: usize,
    pub logged_in: LoginState,
    pub challenges: ChallengeCollection,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            counter: 1,
            logged_in: LoginState::LoggedOut,
            challenges: ChallengeCollection::default(),
        }
    }
}

impl AppState {
    pub fn set_logged_in(app_state: &Arc<Mutex<AppState>>) {
        let app = Arc::clone(app_state);
        let mut app = app.lock().unwrap();
        app.logged_in = LoginState::LoggedIn;
    }
    pub fn set_logged_out(app_state: &Arc<Mutex<AppState>>) {
        let app = Arc::clone(app_state);
        let mut app = app.lock().unwrap();
        app.logged_in = LoginState::LoggedOut;
    }
}
