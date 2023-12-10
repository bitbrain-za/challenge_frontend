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
    pub last_refresh: chrono::NaiveTime,
    pub last_activity: chrono::NaiveTime,
    pub activity_timeout: chrono::Duration,
    pub refresh_period: chrono::Duration,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            counter: 1,
            logged_in: LoginState::LoggedOut,
            challenges: ChallengeCollection::default(),
            last_refresh: chrono::Utc::now().time(),
            last_activity: chrono::Utc::now().time(),
            activity_timeout: chrono::Duration::minutes(10),
            refresh_period: chrono::Duration::minutes(5),
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

    /* Keep refreshing every refresh period
    until there's been no activity for longer
    than the activity timeout */
    #[allow(dead_code)]
    pub fn needs_refresh(&self) -> bool {
        let now = chrono::Utc::now().time();
        let elapsed_since_activity = now - self.last_activity;
        let elapsed_since_refresh = now - self.last_refresh;
        elapsed_since_refresh > self.refresh_period
            && elapsed_since_activity < self.activity_timeout
    }
    #[allow(dead_code)]
    pub fn update_activity_timer(&mut self) {
        self.last_activity = chrono::Utc::now().time();
    }
}
