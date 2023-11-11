use crate::helpers::refresh;
use gloo_net::http;
use poll_promise::Promise;
use web_sys::RequestCredentials;

#[derive(Clone, Debug)]
pub enum GetStatus {
    NotStarted,
    InProgress,
    Success(String),
    Failed(String),
}

impl std::fmt::Display for GetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GetStatus::NotStarted => write!(f, "Not started"),
            GetStatus::InProgress => write!(f, "Loading..."),
            GetStatus::Success(s) => write!(f, "{}", s),
            GetStatus::Failed(s) => write!(f, "{}", s),
        }
    }
}

enum FetchResponse {
    Success(GetStatus),
    Failure(String),
    FailAuth,
}

pub struct Getter {
    promise: Option<Promise<Result<FetchResponse, String>>>,
    with_credentials: bool,
    url: String,
    retry_count: usize,
    state_has_changed: bool,
    token_refresh_promise: refresh::RefreshPromise,
}

impl Getter {
    pub fn new(url: &str, with_credentials: bool) -> Self {
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
        }
    }

    pub fn check_promise(&mut self) -> GetStatus {
        match refresh::check_refresh_promise(&mut self.token_refresh_promise) {
            refresh::RefreshStatus::NotStarted => {}
            refresh::RefreshStatus::InProgress => {}
            refresh::RefreshStatus::Success => {
                log::debug!("Retrying Request");
                self.get();
                return GetStatus::InProgress;
            }
            refresh::RefreshStatus::Failed(_) => {
                self.state_has_changed = true;
                return GetStatus::Failed("Failed to authenticate".to_string());
            }
        }

        let mut res = GetStatus::NotStarted;
        if let Some(promise) = &self.promise {
            res = match promise.ready() {
                Some(result) => match result {
                    Ok(FetchResponse::Success(status)) => status.clone(),
                    Ok(FetchResponse::Failure(e)) => GetStatus::Failed(e.to_string()),
                    Ok(FetchResponse::FailAuth) => {
                        if self.retry_count > 0 {
                            log::debug!("Retrying auth");
                            self.retry_count -= 1;
                            self.promise = None;
                            self.token_refresh_promise = refresh::submit_refresh();
                            GetStatus::InProgress
                        } else {
                            GetStatus::Failed("Authentication failed".to_string())
                        }
                    }

                    Err(e) => GetStatus::Failed(e.to_string()),
                },
                None => GetStatus::InProgress,
            };
            self.state_has_changed = true;
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

impl Getter {
    pub fn get(&mut self) {
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
                200 => FetchResponse::Success(GetStatus::Success(text)),
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
