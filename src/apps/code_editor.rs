use crate::helpers::{
    fetchers::Requestor,
    submission::{Submission, SubmissionResult},
    AppState, Languages,
};
use std::borrow::BorrowMut;
use std::sync::{Arc, Mutex};

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct CodeEditor {
    #[serde(skip)]
    url: String,
    #[serde(skip)]
    run: Submission,
    #[serde(skip)]
    last_result: SubmissionResult,
    #[serde(skip)]
    code: String,
    #[serde(skip)]
    submitter: Option<Requestor>,
    #[serde(skip)]
    app_state: Arc<Mutex<AppState>>,
}

impl Default for CodeEditor {
    fn default() -> Self {
        let mut run = Submission {
            code: Some("#A very simple example\nprint(\"Hello world!\")".into()),
            ..Default::default()
        };
        run.language = Languages::Python;
        Self {
            url: option_env!("BACKEND_URL")
                .unwrap_or("http://123.4.5.6:3000/")
                .to_string(),
            run,
            code: "#A very simple example\nprint(\"Hello world!\")".into(),
            last_result: SubmissionResult::NotStarted,
            submitter: None,
            app_state: Arc::new(Mutex::new(AppState::default())),
        }
    }
}

impl CodeEditor {
    fn submit(&mut self) {
        let submission = self.run.clone();
        let url = format!("{}api/game/submit", self.url);
        let app_state = Arc::clone(&self.app_state);
        self.submitter = submission.sender(app_state, &url);
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

    fn set_app_state_ref(&mut self, app_state: Arc<Mutex<AppState>>) {
        self.app_state = app_state;
    }

    fn show(&mut self, ctx: &egui::Context, open: &mut bool) {
        use super::View as _;
        egui::Window::new(self.name())
            .open(open)
            .default_height(500.0)
            .show(ctx, |ui| self.ui(ui));

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
            .selected_text(self.run.challenge.clone().unwrap_or("None".to_string()))
            .show_ui(ui, |ui| {
                ui.style_mut().wrap = Some(false);
                ui.set_min_width(60.0);

                for challenge in self.app_state.lock().unwrap().challenges.items.iter() {
                    ui.selectable_value(
                        &mut self.run.challenge,
                        Some(challenge.command.clone()),
                        &challenge.command,
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
                    self.app_state.lock().unwrap().update_activity_timer();
                    self.as_submission();
                    match self.run.validate() {
                        Ok(_) => {
                            self.submit();
                        }
                        Err(e) => {
                            self.last_result = SubmissionResult::Failure { message: e };
                        }
                    }
                }
                if ui.button("Test").clicked() {
                    log::debug!("Testing code");
                    self.app_state.lock().unwrap().update_activity_timer();
                    self.as_test_submission();
                    match self.run.validate() {
                        Ok(_) => {
                            self.submit();
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
