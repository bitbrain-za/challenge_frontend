use gloo_net::http;
use poll_promise::Promise;
use web_sys::RequestCredentials;

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct RefreshResponse {
    pub status: String,
    pub message: String,
}
pub enum RefreshStatus {
    NotStarted,
    InProgress,
    Success,
    Failed(String),
}

pub type RefreshPromise = Option<Promise<Result<RefreshResponse, String>>>;

pub fn submit_refresh(url: &str) -> RefreshPromise {
    let url = format!("{}api/auth/refresh", url);
    log::debug!("Refreshing token");

    Some(Promise::spawn_local(async move {
        let response = http::Request::get(&url)
            .credentials(RequestCredentials::Include)
            .send()
            .await
            .unwrap();
        response
            .json::<RefreshResponse>()
            .await
            .map_err(|e| e.to_string())
    }))
}

pub fn check_refresh_promise(promise: &mut RefreshPromise) -> RefreshStatus {
    let mut res = RefreshStatus::NotStarted;
    if let Some(p) = promise {
        res = RefreshStatus::InProgress;
        if let Some(result) = p.ready() {
            if let Ok(result) = result {
                if "success" == result.status {
                    log::info!("Token refreshed");
                    res = RefreshStatus::Success;
                } else {
                    log::error!("Failed to refresh token: {:?}", result);
                    res = RefreshStatus::Failed(result.message.clone());
                }
            }
            *promise = None;
        }
    }
    res
}
