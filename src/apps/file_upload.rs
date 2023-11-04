use crate::helpers::{
    submission::{Submission, SubmissionResult},
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
pub struct FileUpload {
    #[serde(skip)]
    promise: Option<Promise<Result<SubmissionResult, String>>>,
    #[serde(skip)]
    url: String,
    #[serde(skip)]
    run: Submission,
    #[serde(skip)]
    binary_channel: (Sender<Binary>, Receiver<Binary>),
    #[serde(skip)]
    file: Vec<u8>,
    #[serde(skip)]
    submit: bool,
}

impl Default for FileUpload {
    fn default() -> Self {
        Self {
            promise: Default::default(),
            url: option_env!("BACKEND_URL")
                .unwrap_or("http://123.4.5.6:3000/")
                .to_string(),
            run: Submission::default(),
            binary_channel: channel(),
            submit: false,
            file: vec![],
        }
    }
}

impl FileUpload {
    fn _submit(&mut self, ctx: &egui::Context) {
        if !self.submit {
            return;
        }
        self.submit = false;
        let submission = self.run.clone();

        let url = format!("{}api/game/binary", self.url);
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

        if let Ok(f) = self.binary_channel.1.try_recv() {
            self.run.filename = f.filename;
            self.file = f.bytes;
        }
    }
}

impl super::View for FileUpload {
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

        ui.label(&self.run.filename);
        // a simple button opening the dialog
        if ui.button("Open text file").clicked() {
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
