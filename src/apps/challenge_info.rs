use crate::helpers::{AppState, Challenges};
use egui_commonmark::*;
use std::sync::{Arc, Mutex};

#[derive(PartialEq, Clone, Copy, serde::Deserialize, serde::Serialize)]
enum FilterOption {
    All,
    UniquePlayers,
    UniqueLanguage,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct ChallengeInfoApp {
    selected_challenge: Challenges,
    #[serde(skip)]
    active_challenge: Challenges,
    instructions: String,
    #[serde(skip)]
    app_state: Arc<Mutex<AppState>>,
}

impl Default for ChallengeInfoApp {
    fn default() -> Self {
        Self {
            selected_challenge: Challenges::default(),
            active_challenge: Challenges::None,
            instructions: "None".to_string(),
            app_state: Arc::new(Mutex::new(AppState::default())),
        }
    }
}

impl super::App for ChallengeInfoApp {
    fn name(&self) -> &'static str {
        "📖 Challenge Info"
    }

    fn set_app_state_ref(&mut self, app_state: Arc<Mutex<AppState>>) {
        self.app_state = app_state;
    }

    fn show(&mut self, ctx: &egui::Context, open: &mut bool) {
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

        egui::Window::new(self.name())
            .open(open)
            .default_width(800.0)
            .default_height(600.0)
            .vscroll(false)
            .hscroll(false)
            .resizable(true)
            .constrain(true)
            .collapsible(true)
            .show(ctx, |ui| {
                use super::View as _;
                self.ui(ui);
            });
    }
}

impl super::View for ChallengeInfoApp {
    fn ui(&mut self, ui: &mut egui::Ui) {
        egui::SidePanel::right("ChallengeInfoSelection")
            .resizable(false)
            .show_inside(ui, |ui| {
                ui.vertical(|ui| {
                    for challenge in Challenges::iter() {
                        ui.radio_value(
                            &mut self.selected_challenge,
                            challenge,
                            format!("{}", challenge),
                        );
                    }
                    ui.separator();
                    if ui.button("Refresh").clicked() {
                        self.active_challenge = Challenges::None;
                    }
                });
            });
        egui::CentralPanel::default().show_inside(ui, |ui| {
            egui::ScrollArea::both()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    let mut cache = CommonMarkCache::default();
                    CommonMarkViewer::new("viewer").show(ui, &mut cache, &self.instructions);
                });
        });
    }
}
