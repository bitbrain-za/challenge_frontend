use crate::helpers::{
    submission::{Submission, SubmissionResult},
    Challenges, Languages,
};
use gloo_net::http;
use poll_promise::Promise;
use web_sys::RequestCredentials;

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct CodeEditor {
    #[serde(skip)]
    promise: Option<Promise<Result<SubmissionResult, String>>>,
    #[serde(skip)]
    url: String,
    #[serde(skip)]
    run: Submission,
    #[serde(skip)]
    submit: bool,
    #[serde(skip)]
    code: String,
}

impl Default for CodeEditor {
    fn default() -> Self {
        let mut run = Submission {
            code: Some("#A very simple example\nprint(\"Hello world!\")".into()),
            ..Default::default()
        };
        run.language = Languages::Python;
        Self {
            promise: Default::default(),
            url: option_env!("BACKEND_URL")
                .unwrap_or("http://123.4.5.6:3000/")
                .to_string(),
            run,
            code: "#A very simple example\nprint(\"Hello world!\")".into(),
            submit: false,
        }
    }
}

impl CodeEditor {
    fn submit(&mut self, ctx: &egui::Context) {
        if !self.submit {
            return;
        }
        self.submit = false;
        let submission = self.run.clone();

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

    fn as_test_submission(&mut self) {
        self.run.code = Some(self.code.clone());
        self.run.test = true;
    }

    fn as_submission(&mut self) {
        self.run.code = Some(self.code.clone());
        self.run.test = false;
    }
}

impl super::App for CodeEditor {
    fn name(&self) -> &'static str {
        "💻 Code Editor"
    }

    fn show(&mut self, ctx: &egui::Context, open: &mut bool) {
        self.submit(ctx);
        use super::View as _;
        egui::Window::new(self.name())
            .open(open)
            .default_height(500.0)
            .show(ctx, |ui| self.ui(ui));
    }
}

impl super::View for CodeEditor {
    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.set_height(0.0);
        });

        ui.horizontal(|ui| {
            ui.label("Language:");

            for l in Languages::iter() {
                ui.selectable_value(&mut self.run.language, l, format!("{}", l));
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

        let mut theme = egui_extras::syntax_highlighting::CodeTheme::from_memory(ui.ctx());
        ui.collapsing("Theme", |ui| {
            ui.group(|ui| {
                theme.ui(ui);
                theme.clone().store_in_memory(ui.ctx());
            });
        });

        let mut layouter = |ui: &egui::Ui, string: &str, wrap_width: f32| {
            let mut layout_job = egui_extras::syntax_highlighting::highlight(
                ui.ctx(),
                &theme,
                string,
                &self.run.language.to_string(),
            );
            layout_job.wrap.max_width = wrap_width;
            ui.fonts(|f| f.layout_job(layout_job))
        };

        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.add(
                egui::TextEdit::multiline(&mut self.code)
                    .font(egui::TextStyle::Monospace) // for cursor height
                    .code_editor()
                    .desired_rows(10)
                    .lock_focus(true)
                    .desired_width(f32::INFINITY)
                    .layouter(&mut layouter),
            );
        });
        ui.separator();
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                if ui.button("Submit").clicked() {
                    log::debug!("Submitting code");
                    self.as_submission();
                    self.submit = true;
                }
                if ui.button("Test").clicked() {
                    log::debug!("Testing code");
                    self.as_test_submission();
                    self.submit = true;
                }
            });
            ui.separator();
            ui.vertical(|ui| {
                if let Some(promise) = &self.promise {
                    if let Some(result) = promise.ready() {
                        match result {
                            Ok(submission_response) => match &submission_response {
                                SubmissionResult::Success { score, message } => {
                                    ui.label(format!("Message: {}", message));
                                    ui.label(format!("Score: {}", score));
                                }
                                SubmissionResult::Failure { message } => {
                                    ui.label(format!("Message: {}", message));
                                }
                            },
                            Err(error) => {
                                log::error!("Failed to fetch scores: {}", error);
                            }
                        }
                    } else {
                        ui.label("Running...");
                    }
                }
            });
        });
    }
}
