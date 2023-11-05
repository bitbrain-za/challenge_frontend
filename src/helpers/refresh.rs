use gloo_net::http;
use poll_promise::Promise;
use web_sys::RequestCredentials;

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct RefreshResponse {
    pub status: String,
    pub message: String,
}

pub fn submit_refresh(url: &str) -> Promise<Result<RefreshResponse, String>> {
    let url = format!("{}api/auth/refresh", url);
    log::debug!("Refreshing token");

    Promise::spawn_local(async move {
        let response = http::Request::get(&url)
            .credentials(RequestCredentials::Include)
            .send()
            .await
            .unwrap();
        let result = response
            .json::<RefreshResponse>()
            .await
            .map_err(|e| e.to_string());
        log::info!("Result: {:?}", result);

        result
    })
}
