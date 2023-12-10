use crate::helpers::{
    fetchers::Requestor,
    submission::{Submission, SubmissionResult},
    AppState, Challenges, Languages,
};
use egui::*;
use egui_commonmark::*;
use egui_notify::Toasts;
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[derive(serde::Deserialize, serde::Serialize)]
pub struct CodeEditor {
    code: String,
    show_instructions: bool,
    run: Submission,
    theme: egui_extras::syntax_highlighting::CodeTheme,
    #[serde(skip)]
    instructions: String,
    label: String,

    #[serde(skip)]
    url: String,
    #[serde(skip)]
    last_result: SubmissionResult,
    #[serde(skip)]
    toasts: Toasts,
    #[serde(skip)]
    active_challenge: Challenges,
    #[serde(skip)]
    selected_challenge: Challenges,

    #[serde(skip)]
    submitter: Option<Requestor>,
    #[serde(skip)]
    pub app_state: Arc<Mutex<AppState>>,
}

impl Default for CodeEditor {
    fn default() -> Self {
        Self {
            code: DEFAULT_CODE.trim().to_owned(),
            show_instructions: true,
            run: Default::default(),
            theme: egui_extras::syntax_highlighting::CodeTheme::default(),
            instructions: "No Challenge Loaded".into(),
            label: "Code Editor".into(),

            url: option_env!("BACKEND_URL")
                .unwrap_or("http://123.4.5.6:3000/")
                .to_string(),
            last_result: SubmissionResult::NotStarted,
            toasts: Toasts::default(),
            submitter: None,
            active_challenge: Challenges::None,
            selected_challenge: Challenges::default(),
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
}

impl CodeEditor {
    pub fn panels(&mut self, ctx: &egui::Context) {
        let submission = Submission::check_sender(&mut self.submitter);
        match submission {
            SubmissionResult::NotStarted => {}
            SubmissionResult::Success { score: _, message } => {
                self.toasts
                    .info(format!("Result: {}", message))
                    .set_duration(Some(Duration::from_secs(5)));
            }
            _ => {
                self.last_result = submission;
            }
        }

        self.toasts.show(ctx);

        egui::TopBottomPanel::bottom("code_editor_bottom").show(ctx, |_ui| {
            let _layout = egui::Layout::top_down(egui::Align::Center).with_main_justify(true);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            self.ui(ui);
        });
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) {
        if self.active_challenge != self.selected_challenge {
            self.active_challenge = self.selected_challenge;
            self.instructions = self
                .app_state
                .lock()
                .unwrap()
                .challenges
                .get_instructions(self.selected_challenge)
                .unwrap_or("Unable to load instructions".to_string());
        }

        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                let _ = ui.button("Hotkeys").on_hover_ui(nested_hotkeys_ui);
                ui.checkbox(&mut self.show_instructions, "Show Instructions");

                ui.collapsing("Theme", |ui| {
                    ui.group(|ui| {
                        self.theme.ui(ui);
                    });
                });
            });
            ui.end_row();

            ui.horizontal(|ui| {
                egui::ComboBox::from_label("Challenge")
                    .selected_text(format!("{}", self.selected_challenge))
                    .show_ui(ui, |ui| {
                        ui.style_mut().wrap = Some(false);
                        ui.set_min_width(60.0);

                        for challenge in Challenges::iter() {
                            ui.selectable_value(
                                &mut self.selected_challenge,
                                challenge,
                                format!("{}", challenge),
                            );
                        }
                    });

                ui.separator();

                egui::ComboBox::from_label("Language")
                    .selected_text(format!("{}", self.run.language))
                    .show_ui(ui, |ui| {
                        for l in Languages::iter() {
                            ui.selectable_value(&mut self.run.language, l, format!("{}", l));
                        }
                    });
            });

            ui.horizontal(|ui| {
                ui.horizontal(|ui| {
                    ui.set_height(0.0);

                    ui.label("Filename:");
                    ui.add(
                        egui::widgets::text_edit::TextEdit::singleline(&mut self.run.filename)
                            .char_limit(32),
                    )
                    .on_hover_text("What would you like this to be called on the scoreboard?");
                });
                ui.separator();
                if ui.button("Submit").clicked() {
                    log::debug!("Submitting code");
                    self.run.test = false;
                    self.run.code = Some(self.code.clone());
                    self.run.challenge = self.selected_challenge;
                    match self.run.validate() {
                        Ok(_) => {
                            self.submit();
                        }
                        Err(e) => {
                            self.toasts
                                .error(format!("Invalid Submission: {}", e))
                                .set_duration(Some(Duration::from_secs(5)));
                            self.last_result = SubmissionResult::Failure { message: e };
                        }
                    }
                }
                if ui.button("Test").clicked() {
                    self.run.test = true;
                    self.run.code = Some(self.code.clone());
                    self.run.challenge = self.selected_challenge;
                    match self.run.validate() {
                        Ok(_) => {
                            log::debug!("Testing code");
                            self.submit();
                        }
                        Err(e) => {
                            self.toasts
                                .error(format!("Invalid Submission: {}", e))
                                .set_duration(Some(Duration::from_secs(5)));

                            log::error!("Validation Error: {}", e);
                            self.last_result = SubmissionResult::Failure { message: e };
                        }
                    }
                }
            });
            ui.separator();
        });

        if self.show_instructions {
            ui.columns(2, |columns| {
                ScrollArea::vertical()
                    .id_source("source")
                    .show(&mut columns[0], |ui| self.editor_ui(ui));
                ScrollArea::vertical()
                    .id_source("rendered")
                    .show(&mut columns[1], |ui| self.instructions_ui(ui));
            });
        } else {
            ScrollArea::vertical()
                .id_source("source")
                .show(ui, |ui| self.editor_ui(ui));
        }
    }

    fn editor_ui(&mut self, ui: &mut egui::Ui) {
        let mut layouter = |ui: &egui::Ui, string: &str, wrap_width: f32| {
            let mut layout_job = egui_extras::syntax_highlighting::highlight(
                ui.ctx(),
                &self.theme,
                string,
                &self.run.language.to_string(),
            );
            layout_job.wrap.max_width = wrap_width;
            ui.fonts(|f| f.layout_job(layout_job))
        };

        ui.add(
            egui::TextEdit::multiline(&mut self.code)
                .font(egui::TextStyle::Monospace) // for cursor height
                .code_editor()
                .desired_rows(10)
                .lock_focus(true)
                .desired_width(f32::INFINITY)
                .layouter(&mut layouter),
        );
    }

    fn instructions_ui(&mut self, ui: &mut egui::Ui) {
        let mut cache = CommonMarkCache::default();
        CommonMarkViewer::new("viewer").show(ui, &mut cache, &self.instructions);
    }
}

pub const SHORTCUT_TEST: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::T);
pub const SHORTCUT_SUBMIT: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::R);

fn nested_hotkeys_ui(ui: &mut egui::Ui) {
    egui::Grid::new("shortcuts").striped(true).show(ui, |ui| {
        let mut label = |shortcut, what| {
            ui.label(what);
            ui.weak(ui.ctx().format_shortcut(&shortcut));
            ui.end_row();
        };

        label(SHORTCUT_TEST, "Test");
        label(SHORTCUT_SUBMIT, "Submit");
    });
}

// ----------------------------------------------------------------------------

const DEFAULT_CODE: &str = r#"
import json
import sys

def main():
    for line in sys.stdin:
        if line == "q\n": break
        if line == "\n":
            sys.stdout.write("0")
            sys.stdout.write("\n")
            sys.stdout.flush()
            continue
        input_ints = line.rstrip().split(',')
        answer = find_the_number(input_ints)
        sys.stdout.write(answer)
        sys.stdout.write("\n")
        sys.stdout.flush()

def find_the_number(int_list):
    unique_ints = set(int_list)
    for integer in unique_ints:
        if int_list.count(integer) % 2 != 0:
            # print("the number that appears an odd number of times is", integer)
            return integer


main()
"#;
