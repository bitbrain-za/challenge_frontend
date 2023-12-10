use crate::helpers::{refresh, AppState};
use std::sync::{Arc, Mutex};

#[derive(PartialEq)]
enum State {
    Idle,
    Fetching,
    Error,
}

pub struct LoginFetcher {
    app_state: Arc<Mutex<AppState>>,
    state: State,
    token_refresh_promise: refresh::RefreshPromise,
}

impl Default for LoginFetcher {
    fn default() -> Self {
        Self {
            app_state: Arc::new(Mutex::new(AppState::default())),
            state: State::Idle,
            token_refresh_promise: None,
        }
    }
}

impl LoginFetcher {
    pub fn new(app_state: Arc<Mutex<AppState>>) -> Self {
        Self {
            app_state: app_state.clone(),
            state: State::Idle,
            token_refresh_promise: None,
        }
    }

    pub fn tick(&mut self) {
        self.fetch();
        self.check_info_promise();
    }

    fn fetch(&mut self) {
        if !self.app_state.lock().unwrap().needs_refresh() {
            return;
        }
        self.app_state.lock().unwrap().last_refresh = chrono::Utc::now().time();
        log::debug!("Refreshing token");
        self.state = State::Fetching;
        self.token_refresh_promise = refresh::submit_refresh();
    }

    fn check_info_promise(&mut self) {
        match refresh::check_refresh_promise(&mut self.token_refresh_promise) {
            refresh::RefreshStatus::NotStarted => {}
            refresh::RefreshStatus::InProgress => {}
            refresh::RefreshStatus::Success => {
                AppState::set_logged_in(&self.app_state);
                self.state = State::Idle;
            }
            refresh::RefreshStatus::Failed(_) => {
                AppState::set_logged_out(&self.app_state);
                self.state = State::Error;
            }
        }
    }
}
