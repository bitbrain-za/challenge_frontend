use std::borrow::BorrowMut;
use std::sync::{Arc, Mutex};

use crate::helpers::{
    fetchers::Requestor,
    submission::{Submission, SubmissionResult},
    AppState, Challenges, Languages,
};
use std::future::Future;
use std::sync::mpsc::{channel, Receiver, Sender};

struct Binary {
    filename: String,
    bytes: Vec<u8>,
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct BinaryUpload {
    #[serde(skip)]
    last_result: SubmissionResult,
    url: String,
    #[serde(skip)]
    run: Submission,
    #[serde(skip)]
    binary_channel: (Sender<Binary>, Receiver<Binary>),
    #[serde(skip)]
    submitter: Option<Requestor>,
    #[serde(skip)]
    app_state: Arc<Mutex<AppState>>,
}

impl Default for BinaryUpload {
    fn default() -> Self {
        Self {
            url: option_env!("BACKEND_URL")
                .unwrap_or("http://123.4.5.6:3000/")
                .to_string(),
            run: Submission {
                filename: "Select Binary".to_string(),
                ..Default::default()
            },
            binary_channel: channel(),
            submitter: None,
            last_result: SubmissionResult::NotStarted,
            app_state: Arc::new(Mutex::new(AppState::default())),
        }
    }
}

impl BinaryUpload {
    fn submit(&mut self) {
        let submission = self.run.clone();
        let url = format!("{}api/game/submit", self.url);
        let app_state = Arc::clone(&self.app_state);
        self.submitter = submission.sender(app_state, &url);
    }
}

impl super::App for BinaryUpload {
    fn name(&self) -> &'static str {
        "💻 File Upload"
    }

    fn set_app_state_ref(&mut self, app_state: Arc<Mutex<AppState>>) {
        self.app_state = app_state;
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

        let submission = Submission::check_sender(&mut self.submitter);
        match submission {
            SubmissionResult::NotStarted => {}
            _ => {
                self.last_result = submission;
            }
        }
        if let Some(fetcher) = self.submitter.borrow_mut() {
            if fetcher.refresh_context() {
                ctx.request_repaint();
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
                        self.submit();
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
