use gloo_net::http;
use poll_promise::Promise;
use serde::de::DeserializeOwned;
use std::marker::{Send, Sync};
use web_sys::RequestCredentials;

pub enum GetStatus<T: Clone + Sync + Send> {
    NotStarted,
    InProgress,
    Success(T),
    Failed(String),
}

impl std::fmt::Display for GetStatus<String> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GetStatus::NotStarted => write!(f, "Not started"),
            GetStatus::InProgress => write!(f, "Loading..."),
            GetStatus::Success(s) => write!(f, "{}", s),
            GetStatus::Failed(s) => write!(f, "{}", s),
        }
    }
}

pub struct Getter<T: Clone + Sync + Send + 'static> {
    promise: Option<Promise<Result<T, String>>>,
    with_credentials: bool,
    context: Option<egui::Context>,
    url: String,
}

impl<T: Clone + Sync + Send> Getter<T> {
    pub fn new(url: &str, ctx: Option<&egui::Context>, with_credentials: bool) -> Self {
        Self {
            url: url.to_string(),
            promise: None,
            with_credentials,
            context: ctx.cloned(),
        }
    }

    pub fn check_promise(&mut self) -> GetStatus<T> {
        let mut res = GetStatus::<T>::NotStarted;
        if let Some(promise) = &self.promise {
            res = match promise.ready() {
                Some(result) => match result {
                    Ok(response) => GetStatus::Success(response.clone()),
                    Err(e) => GetStatus::Failed(e.to_string()),
                },
                None => GetStatus::InProgress,
            };
        }
        res
    }
}

impl Getter<String> {
    pub fn get(&mut self) {
        let url = self.url.clone();
        let ctx = self.context.clone();
        let with_credentials = self.with_credentials;
        let promise = Promise::spawn_local(async move {
            let request = http::Request::get(&url);
            let request = match with_credentials {
                true => request.credentials(RequestCredentials::Include),
                false => request,
            };
            let response = request.send().await.map_err(|e| e.to_string())?;
            if let Some(ctx) = ctx {
                ctx.request_repaint(); // wake up UI thread
            }
            match response.text().await {
                Ok(text) => Ok(text),
                Err(e) => Err(e.to_string()),
            }
        });
        self.promise = Some(promise);
    }
}

impl<T: Clone + DeserializeOwned + Sync + Send> Getter<T> {
    pub fn get_json(&mut self) {
        let url = self.url.clone();
        let ctx = self.context.clone();
        let with_credentials = self.with_credentials;
        let promise = Promise::spawn_local(async move {
            let request = http::Request::get(&url);
            let request = match with_credentials {
                true => request.credentials(RequestCredentials::Include),
                false => request,
            };
            let response = request.send().await.map_err(|e| e.to_string())?;
            if let Some(ctx) = ctx {
                ctx.request_repaint(); // wake up UI thread
            }
            response.json::<T>().await.map_err(|e| e.to_string())
        });
        self.promise = Some(promise);
    }
}
