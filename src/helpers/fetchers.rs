use crate::helpers::{refresh, AppState};
use gloo_net::http;
use poll_promise::Promise;
use std::sync::{Arc, Mutex};
use web_sys::{FormData, RequestCredentials};

#[derive(Clone, Debug)]
pub enum RequestStatus {
    NotStarted,
    InProgress,
    Success(String),
    Failed(String),
}

enum Method {
    Get,
    Post,
    Refresh,
}

impl std::fmt::Display for RequestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RequestStatus::NotStarted => write!(f, "Not started"),
            RequestStatus::InProgress => write!(f, "Loading..."),
            RequestStatus::Success(s) => write!(f, "{}", s),
            RequestStatus::Failed(s) => write!(f, "{}", s),
        }
    }
}

enum FetchResponse {
    Success(RequestStatus),
    Failure(String),
    FailAuth,
}

pub struct Requestor {
    promise: Option<Promise<Result<FetchResponse, String>>>,
    with_credentials: bool,
    url: String,
    retry_count: usize,
    state_has_changed: bool,
    token_refresh_promise: refresh::RefreshPromise,
    post_data: Option<String>,
    form_data: Option<FormData>,
    method: Method,
    pub app_state: Arc<Mutex<AppState>>,
}

impl Requestor {
    pub fn new_get(app_state: Arc<Mutex<AppState>>, url: &str, with_credentials: bool) -> Self {
        Self::new(app_state, url, with_credentials, None, None, Method::Get)
    }

    pub fn new_refresh(app_state: Arc<Mutex<AppState>>) -> Self {
        let mut s = Self::new(app_state, "", true, None, None, Method::Refresh);
        s.refresh_login();
        s
    }

    pub fn new_post(
        app_state: Arc<Mutex<AppState>>,
        url: &str,
        with_credentials: bool,
        data: Option<String>,
    ) -> Self {
        Self::new(app_state, url, with_credentials, data, None, Method::Post)
    }
    pub fn new_form_post(
        app_state: Arc<Mutex<AppState>>,
        url: &str,
        with_credentials: bool,
        data: Option<FormData>,
    ) -> Self {
        Self::new(app_state, url, with_credentials, None, data, Method::Post)
    }

    fn new(
        app_state: Arc<Mutex<AppState>>,
        url: &str,
        with_credentials: bool,
        data: Option<String>,
        form: Option<FormData>,
        method: Method,
    ) -> Self {
        Self {
            url: url.to_string(),
            promise: None,
            with_credentials,
            retry_count: match with_credentials {
                true => 1,
                false => 0,
            },
            state_has_changed: false,
            token_refresh_promise: None,
            post_data: data,
            form_data: form,
            method,
            app_state,
        }
    }

    pub fn check_promise(&mut self) -> RequestStatus {
        match refresh::check_refresh_promise(&mut self.token_refresh_promise) {
            refresh::RefreshStatus::NotStarted => {}
            refresh::RefreshStatus::InProgress => {}
            refresh::RefreshStatus::Success => {
                log::debug!("Retrying Request");
                AppState::set_logged_in(&self.app_state);
                self.state_has_changed = true;
                self.send();
                return RequestStatus::InProgress;
            }
            refresh::RefreshStatus::Failed(_) => {
                self.state_has_changed = true;
                return RequestStatus::Failed("Failed to authenticate".to_string());
            }
        }

        let mut res = RequestStatus::NotStarted;
        if let Some(promise) = &self.promise {
            res = match promise.ready() {
                Some(result) => match result {
                    Ok(FetchResponse::Success(status)) => status.clone(),
                    Ok(FetchResponse::Failure(e)) => RequestStatus::Failed(e.to_string()),
                    Ok(FetchResponse::FailAuth) => {
                        if self.retry_count > 0 {
                            log::debug!("Retrying auth");
                            self.retry_count -= 1;
                            self.token_refresh_promise = refresh::submit_refresh();
                            RequestStatus::InProgress
                        } else {
                            AppState::set_logged_out(&self.app_state);
                            RequestStatus::Failed("Authentication failed".to_string())
                        }
                    }

                    Err(e) => RequestStatus::Failed(e.to_string()),
                },
                None => RequestStatus::InProgress,
            };
            self.state_has_changed = true;
        }
        match res {
            RequestStatus::InProgress => {}
            _ => {
                self.promise = None;
            }
        }
        res
    }

    pub fn refresh_context(&mut self) -> bool {
        match self.state_has_changed {
            true => {
                self.state_has_changed = false;
                true
            }
            false => false,
        }
    }
}

impl Requestor {
    pub fn send(&mut self) {
        match self.method {
            Method::Get => self.get(),
            Method::Post => self.post(),
            Method::Refresh => (),
        }
    }

    fn refresh_login(&mut self) {
        self.token_refresh_promise = refresh::submit_refresh();
    }

    fn get(&mut self) {
        let url = self.url.clone();
        let with_credentials = self.with_credentials;
        let promise = Promise::spawn_local(async move {
            let request = http::Request::get(&url);
            let request = match with_credentials {
                true => request.credentials(RequestCredentials::Include),
                false => request,
            };
            let response = request.send().await.map_err(|e| e.to_string())?;
            let text = response.text().await.map_err(|e| e.to_string())?;

            let result = match response.status() {
                200 => FetchResponse::Success(RequestStatus::Success(text)),
                401 => {
                    log::warn!("Auth Error: {}", text);
                    FetchResponse::FailAuth
                }
                _ => {
                    log::error!("Response: {}", text);
                    FetchResponse::Failure(text)
                }
            };
            Ok(result)
        });
        self.promise = Some(promise);
    }

    fn post(&mut self) {
        let url = self.url.clone();
        let with_credentials = self.with_credentials;
        let json_data = self.post_data.clone();
        let form_data = self.form_data.clone();

        let promise = Promise::spawn_local(async move {
            let request = http::Request::post(&url);
            let request = match with_credentials {
                true => request.credentials(RequestCredentials::Include),
                false => request,
            };
            let request = {
                if let Some(data) = json_data {
                    request
                        .header("Content-Type", "application/json")
                        .body(data)
                        .map_err(|e| e.to_string())?
                } else if let Some(data) = form_data {
                    request.body(data).map_err(|e| e.to_string())?
                } else {
                    request.build().map_err(|e| e.to_string())?
                }
            };

            let response = request.send().await.map_err(|e| e.to_string())?;
            let text = response.text().await.map_err(|e| e.to_string())?;

            let result = match response.status() {
                200 => FetchResponse::Success(RequestStatus::Success(text)),
                401 => {
                    log::warn!("Auth Error: {}", text);
                    FetchResponse::FailAuth
                }
                _ => {
                    log::error!("Response: {}", text);
                    FetchResponse::Failure(text)
                }
            };
            Ok(result)
        });
        self.promise = Some(promise);
    }
}
