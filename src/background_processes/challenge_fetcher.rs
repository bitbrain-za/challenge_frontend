use crate::helpers::{
    fetchers::{RequestStatus, Requestor},
    AppState, ChallengeCollection,
};
use std::sync::{Arc, Mutex};

#[derive(PartialEq, Clone, Copy)]
enum State {
    Dirty,
    Fetching,
    Clean,
}

pub struct ChallengeFetcher {
    state: State,
    info_fetcher: Option<Requestor>,
    app_state: Arc<Mutex<AppState>>,
}

impl Default for ChallengeFetcher {
    fn default() -> Self {
        Self {
            info_fetcher: None,
            state: State::Dirty,
            app_state: Arc::new(Mutex::new(AppState::default())),
        }
    }
}

impl ChallengeFetcher {
    pub fn new(app_state: Arc<Mutex<AppState>>) -> Self {
        Self {
            info_fetcher: None,
            state: State::Dirty,
            app_state: app_state.clone(),
        }
    }

    pub fn tick(&mut self) {
        self.fetch();
        self.check_info_promise();
    }

    fn fetch(&mut self) {
        if self.state != State::Dirty {
            return;
        }
        log::debug!("Fetching challenge info");
        self.state = State::Fetching;
        let app_state = self.app_state.clone();
        let my_app_state = self.app_state.clone();
        self.info_fetcher = my_app_state.lock().unwrap().challenges.fetch(app_state);
    }
    fn check_info_promise(&mut self) {
        if self.state != State::Fetching {
            return;
        }

        let getter = &mut self.info_fetcher;

        if let Some(getter) = getter {
            let result = &getter.check_promise();
            match result {
                RequestStatus::NotStarted => {}
                RequestStatus::InProgress => {}
                RequestStatus::Success(data) => {
                    log::debug!("Challenge info fetch success: {}", data);
                    self.info_fetcher = None;
                    self.state = State::Clean;
                }
                RequestStatus::Failed(_) => {
                    self.info_fetcher = None;
                    self.state = State::Dirty;
                }
            }
            self.app_state.lock().unwrap().challenges =
                ChallengeCollection::from_json(&result.to_string());
        }
    }
}
