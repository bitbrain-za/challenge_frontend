use crate::helpers::{submission::Submission, Challenges, Languages};
use egui::*;

#[derive(serde::Deserialize, serde::Serialize)]
pub struct CodeEditor {
    code: String,
    show_instructions: bool,
    run: Submission,
    theme: egui_extras::syntax_highlighting::CodeTheme,
}

impl Default for CodeEditor {
    fn default() -> Self {
        Self {
            code: DEFAULT_CODE.trim().to_owned(),
            show_instructions: false,
            run: Default::default(),
            theme: egui_extras::syntax_highlighting::CodeTheme::default(),
        }
    }
}

impl CodeEditor {
    pub fn panels(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("easy_mark_bottom").show(ctx, |_ui| {
            let _layout = egui::Layout::top_down(egui::Align::Center).with_main_justify(true);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            self.ui(ui);
        });
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) {
        egui::Grid::new("controls").show(ui, |ui| {
            let _ = ui.button("Hotkeys").on_hover_ui(nested_hotkeys_ui);
            ui.checkbox(&mut self.show_instructions, "Show Instructions");

            if ui.button("Submit").clicked() {
                log::debug!("Submitting code");
                todo!();
            }
            if ui.button("Test").clicked() {
                log::debug!("Testing code");
                todo!();
            }
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
            ui.collapsing("Theme", |ui| {
                ui.group(|ui| {
                    self.theme.ui(ui);
                });
            });
            ui.end_row();
        });
        ui.separator();

        if self.show_instructions {
            ui.columns(2, |columns| {
                ScrollArea::vertical()
                    .id_source("source")
                    .show(&mut columns[0], |ui| self.editor_ui(ui));
                ScrollArea::vertical()
                    .id_source("rendered")
                    .show(&mut columns[1], |_ui| {});
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
