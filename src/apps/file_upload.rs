use crate::helpers::{Challenges, Languages};
use gloo_net::http;
use poll_promise::Promise;
use std::future::Future;
use std::sync::mpsc::{channel, Receiver, Sender};
use web_sys::RequestCredentials;

#[derive(Clone, PartialEq, serde::Serialize)]
struct Submission {
    challenge: String,
    player: String,
    name: String,
    language: String,
    code: String,
    test: bool,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
pub enum SubmissionResult {
    Success { score: u32, message: String },
    Failure { message: String },
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct FileUpload {
    language: Languages,
    code: String,
    challenge: Challenges,
    #[serde(skip)]
    promise: Option<Promise<Result<SubmissionResult, String>>>,
    #[serde(skip)]
    url: String,
    #[serde(skip)]
    run: Option<Submission>,
    #[serde(skip)]
    text_channel: (Sender<String>, Receiver<String>),
    #[serde(skip)]
    sample_text: String,
}

impl Default for FileUpload {
    fn default() -> Self {
        Self {
            language: Languages::Python,
            code: "#A very simple example\nprint(\"Hello world!\")".into(),
            promise: Default::default(),
            url: option_env!("BACKEND_URL")
                .unwrap_or("http://123.4.5.6:3000/")
                .to_string(),
            run: None,
            challenge: Challenges::default(),
            text_channel: channel(),
            sample_text: "yo".into(),
        }
    }
}

impl FileUpload {
    fn _submit(&mut self, ctx: &egui::Context) {
        if self.run.is_none() {
            return;
        }
        let submission = self.run.clone().unwrap();
        self.run = None;

        let url = format!("{}api/game/submit", self.url);
        log::debug!("Sending to {}", url);
        let ctx = ctx.clone();

        let promise = Promise::spawn_local(async move {
            let response = http::Request::post(&url)
                .credentials(RequestCredentials::Include)
                .json(&submission)
                .unwrap()
                .send()
                .await
                .unwrap();

            match response.status() {
                200 => (),
                _ => {
                    return Err(format!("Failed to submit code: {:?}", response));
                }
            }

            log::debug!("Response: {:?}", response);
            let headers = response.headers();
            log::debug!("Headers: {:?}", headers);
            for (key, value) in headers.entries() {
                log::debug!("{}: {:?}", key, value);
            }
            let result: SubmissionResult = response.json().await.unwrap();

            ctx.request_repaint(); // wake up UI thread
            log::info!("Result: {:?}", result);
            Ok(result)
        });

        self.promise = Some(promise);
    }

    fn _as_test_submission(&self) -> Submission {
        Submission {
            test: true,
            ..self._as_submission()
        }
    }

    fn _as_submission(&self) -> Submission {
        let challenge = self.challenge.to_string();
        let player = "player".to_string();
        let name = "name".to_string();
        let language = self.language.to_string();
        let code = self.code.clone();
        let test = false;
        Submission {
            challenge,
            player,
            name,
            language,
            code,
            test,
        }
    }
}

impl super::App for FileUpload {
    fn name(&self) -> &'static str {
        "💻 File Upload"
    }

    fn show(&mut self, ctx: &egui::Context, open: &mut bool) {
        use super::View as _;
        egui::Window::new(self.name())
            .open(open)
            .default_height(500.0)
            .show(ctx, |ui| self.ui(ui));

        if let Ok(f) = self.text_channel.1.try_recv() {
            self.sample_text = f;
        }
    }
}

impl super::View for FileUpload {
    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.label(&self.sample_text);
        // a simple button opening the dialog
        if ui.button("Open text file").clicked() {
            let sender = self.text_channel.0.clone();
            let task = rfd::AsyncFileDialog::new().pick_file();
            execute(async move {
                let file = task.await;
                if let Some(file) = file {
                    let text = file.read().await;
                    let _ = sender.send(String::from_utf8_lossy(&text).to_string());
                }
            });
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn execute<F: Future<Output = ()> + Send + 'static>(_f: F) {
    todo!();
}

#[cfg(target_arch = "wasm32")]
fn execute<F: Future<Output = ()> + 'static>(f: F) {
    wasm_bindgen_futures::spawn_local(f);
}
