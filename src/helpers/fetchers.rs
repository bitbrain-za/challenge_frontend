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

pub struct Getter<T: Clone + Sync + Send + 'static> {
    pub promise: Option<Promise<Result<T, String>>>,
}

impl<T: Clone + Sync + Send> Getter<T> {
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
    pub fn get(
        &mut self,
        url: &'static str,
        ctx: Option<&'static egui::Context>,
    ) -> Promise<Result<String, String>> {
        Promise::spawn_local(async move {
            let response = http::Request::get(url)
                .credentials(RequestCredentials::Include)
                .send()
                .await
                .map_err(|e| e.to_string())?;
            if let Some(ctx) = ctx {
                ctx.request_repaint(); // wake up UI thread
            }
            match response.text().await {
                Ok(text) => Ok(text),
                Err(e) => Err(e.to_string()),
            }
        })
    }
}

impl<T: Clone + DeserializeOwned + Sync + Send> Getter<T> {
    pub fn get_json(
        &mut self,
        url: &'static str,
        ctx: Option<&'static egui::Context>,
    ) -> Promise<Result<T, String>> {
        Promise::spawn_local(async move {
            let response = http::Request::get(url)
                .credentials(RequestCredentials::Include)
                .send()
                .await
                .map_err(|e| e.to_string())?;
            if let Some(ctx) = ctx {
                ctx.request_repaint(); // wake up UI thread
            }
            response.json::<T>().await.map_err(|e| e.to_string())
        })
    }
}
