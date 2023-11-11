use crate::helpers::{
    refresh,
    submission::{Submission, SubmissionPromise, SubmissionResult},
    Challenges, Languages,
};
use gloo_net::http;
use poll_promise::Promise;
use std::future::Future;
use std::sync::mpsc::{channel, Receiver, Sender};
use web_sys::RequestCredentials;

struct Binary {
    filename: String,
    bytes: Vec<u8>,
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct BinaryUpload {
    #[serde(skip)]
    promise: SubmissionPromise,
    #[serde(skip)]
    last_result: SubmissionResult,
    url: String,
    #[serde(skip)]
    run: Submission,
    #[serde(skip)]
    binary_channel: (Sender<Binary>, Receiver<Binary>),
    #[serde(skip)]
    submit: bool,
    #[serde(skip)]
    token_refresh_promise: refresh::RefreshPromise,
}

impl Default for BinaryUpload {
    fn default() -> Self {
        Self {
            promise: Default::default(),
            url: option_env!("BACKEND_URL")
                .unwrap_or("http://123.4.5.6:3000/")
                .to_string(),
            run: Submission {
                filename: "Select Binary".to_string(),
                ..Default::default()
            },
            binary_channel: channel(),
            submit: false,
            token_refresh_promise: None,
            last_result: SubmissionResult::NotStarted,
        }
    }
}

impl BinaryUpload {
    fn submit(&mut self, ctx: &egui::Context) {
        if !self.submit {
            return;
        }
        self.submit = false;
        let submission = self.run.clone();

        let url = format!("{}api/game/binary", self.url);
        log::debug!("Sending to {}", url);
        let ctx = ctx.clone();

        let promise = Promise::spawn_local(async move {
            let formdata = submission.to_formdata();

            let response = http::Request::post(&url)
                .credentials(RequestCredentials::Include)
                .body(formdata)
                .unwrap()
                .send()
                .await
                .unwrap();

            let result: SubmissionResult = match response.status() {
                200 => response.json().await.unwrap(),
                401 => {
                    let text = response.text().await;
                    let text = text.map(|text| text.to_owned());
                    let text = match text {
                        Ok(text) => text,
                        Err(e) => e.to_string(),
                    };
                    log::warn!("Auth Error: {:?}", text);
                    SubmissionResult::NotAuthorized
                }
                _ => {
                    return Err(format!("Failed to submit code: {:?}", response));
                }
            };

            ctx.request_repaint(); // wake up UI thread
            log::info!("Result: {:?}", result);
            Ok(result)
        });

        self.promise = Some(promise);
    }
}

impl super::App for BinaryUpload {
    fn name(&self) -> &'static str {
        "💻 File Upload"
    }

    fn show(&mut self, ctx: &egui::Context, open: &mut bool) {
        use super::View as _;
        egui::Window::new(self.name())
            .open(open)
            .default_height(500.0)
            .show(ctx, |ui| self.ui(ui));

        if let Ok(f) = self.binary_channel.1.try_recv() {
            self.run.filename = f.filename;
            self.run.binary = Some(f.bytes);
        }
        self.submit(ctx);
        match refresh::check_refresh_promise(&mut self.token_refresh_promise) {
            refresh::RefreshStatus::InProgress => {}
            refresh::RefreshStatus::Success => {
                self.submit = true;
            }
            refresh::RefreshStatus::Failed(_) => {}
            _ => (),
        }

        let submission = Submission::check_submit_promise(&mut self.promise);
        match submission {
            SubmissionResult::NotStarted => {}
            SubmissionResult::NotAuthorized => {
                self.token_refresh_promise = refresh::submit_refresh();
                self.last_result = submission;
            }
            _ => {
                self.last_result = submission;
            }
        }
    }
}

impl super::View for BinaryUpload {
    fn ui(&mut self, ui: &mut egui::Ui) {
        egui::ComboBox::from_label("Language")
            .selected_text(format!("{}", self.run.language))
            .show_ui(ui, |ui| {
                ui.style_mut().wrap = Some(false);
                ui.set_min_width(60.0);

                for language in Languages::iter() {
                    ui.selectable_value(&mut self.run.language, language, format!("{}", language));
                }
            });

        egui::ComboBox::from_label("Challenge")
            .selected_text(format!("{}", self.run.challenge))
            .show_ui(ui, |ui| {
                ui.style_mut().wrap = Some(false);
                ui.set_min_width(60.0);

                for challenge in Challenges::iter() {
                    ui.selectable_value(
                        &mut self.run.challenge,
                        challenge,
                        format!("{}", challenge),
                    );
                }
            });

        ui.checkbox(&mut self.run.test, "Test");
        ui.separator();

        if ui.button(self.run.filename.clone()).clicked() {
            let sender = self.binary_channel.0.clone();
            let task = rfd::AsyncFileDialog::new().pick_file();
            execute(async move {
                let file = task.await;
                if let Some(file) = file {
                    let bytes = file.read().await;
                    let _ = sender.send(Binary {
                        filename: file.file_name(),
                        bytes,
                    });
                }
            });
        }

        if "Select Binary" != &self.run.filename {
            ui.separator();
            if ui.button("Submit").clicked() {
                match self.run.validate() {
                    Ok(_) => {
                        self.submit = true;
                    }
                    Err(e) => {
                        self.last_result = SubmissionResult::Failure { message: e };
                    }
                }
            }
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
