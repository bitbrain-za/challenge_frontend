use crate::helpers::{
    refresh,
    submission::{Submission, SubmissionPromise, SubmissionResult},
    Challenges, Languages,
};
use gloo_net::http;
use poll_promise::Promise;
use web_sys::RequestCredentials;

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct CodeEditor {
    #[serde(skip)]
    promise: SubmissionPromise,
    #[serde(skip)]
    url: String,
    #[serde(skip)]
    run: Submission,
    #[serde(skip)]
    last_result: SubmissionResult,
    #[serde(skip)]
    submit: bool,
    #[serde(skip)]
    code: String,
    #[serde(skip)]
    token_refresh_promise: refresh::RefreshPromise,
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
            token_refresh_promise: None,
            last_result: SubmissionResult::NotStarted,
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
                self.token_refresh_promise = refresh::submit_refresh(&self.url);
                self.last_result = submission;
            }
            _ => {
                self.last_result = submission;
            }
        }
    }
}

impl super::View for CodeEditor {
    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.set_height(0.0);

            ui.label("Filename:");
            ui.add(egui::widgets::text_edit::TextEdit::singleline(
                &mut self.run.filename,
            ))
            .on_hover_text("What would you like this to be called on the scoreboard?");
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
                    match self.run.validate() {
                        Ok(_) => {
                            self.submit = true;
                        }
                        Err(e) => {
                            self.last_result = SubmissionResult::Failure { message: e };
                        }
                    }
                }
                if ui.button("Test").clicked() {
                    log::debug!("Testing code");
                    self.as_test_submission();
                    match self.run.validate() {
                        Ok(_) => {
                            self.submit = true;
                        }
                        Err(e) => {
                            self.last_result = SubmissionResult::Failure { message: e };
                        }
                    }
                }
            });
            ui.separator();
            ui.vertical(|ui| ui.label(self.last_result.to_string()));
        });
    }
}
