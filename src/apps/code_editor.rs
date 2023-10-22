use crate::helpers::{Challenges, Languages};
use poll_promise::Promise;

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

struct SubmissionResponse {
    _response: ehttp::Response,
    result: SubmissionResult,
}

impl SubmissionResponse {
    fn from_response(_: &egui::Context, response: ehttp::Response) -> Self {
        let _ = response.content_type().unwrap_or_default();
        let text = response.text();
        let text = text.map(|text| text.to_owned());
        log::debug!("Response: {:?}", text);
        let result: SubmissionResult = serde_json::from_str(text.as_ref().unwrap()).unwrap();

        Self {
            _response: response,
            result,
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct CodeEditor {
    language: Languages,
    code: String,
    challenge: Challenges,
    #[serde(skip)]
    promise: Option<Promise<ehttp::Result<SubmissionResponse>>>,
    #[serde(skip)]
    url: String,
    #[serde(skip)]
    run: Option<Submission>,
}

impl Default for CodeEditor {
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
        }
    }
}

impl CodeEditor {
    fn submit(&mut self, ctx: &egui::Context) {
        if self.run.is_none() {
            return;
        }
        let submission = self.run.clone().unwrap();

        let url = format!("{}submit", self.url);
        log::debug!("Sending to {}", url);
        let ctx = ctx.clone();
        let (sender, promise) = Promise::new();

        let submission = serde_json::to_string(&submission).unwrap();
        let request = ehttp::Request::post(url, submission.as_bytes().to_vec());
        ehttp::fetch(request, move |response| {
            ctx.request_repaint(); // wake up UI thread
            let resource =
                response.map(|response| SubmissionResponse::from_response(&ctx, response));
            sender.send(resource);
        });
        self.promise = Some(promise);
        self.run = None;
    }

    fn as_test_submission(&self) -> Submission {
        Submission {
            test: true,
            ..self.as_submission()
        }
    }

    fn as_submission(&self) -> Submission {
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
                ui.selectable_value(&mut self.language, l, format!("{}", l));
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
                &self.language.to_string(),
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
                    self.run = Some(self.as_submission());
                }
                if ui.button("Test").clicked() {
                    log::debug!("Testing code");
                    self.run = Some(self.as_test_submission());
                }
            });
            ui.separator();
            ui.vertical(|ui| {
                if let Some(promise) = &self.promise {
                    if let Some(result) = promise.ready() {
                        match result {
                            Ok(submission_response) => match &submission_response.result {
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
