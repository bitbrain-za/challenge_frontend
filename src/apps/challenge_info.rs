use crate::helpers::AppState;
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
    selected_challenge: String,
    #[serde(skip)]
    active_challenge: Option<String>,
    instructions: String,
    #[serde(skip)]
    app_state: Arc<Mutex<AppState>>,
}

impl Default for ChallengeInfoApp {
    fn default() -> Self {
        Self {
            selected_challenge: "".to_string(),
            active_challenge: None,
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
        let challenges_differ = match self.active_challenge.clone() {
            None => true,
            Some(active) => active != self.selected_challenge,
        };

        if challenges_differ {
            self.active_challenge = Some(self.selected_challenge.clone());
            self.instructions = self
                .app_state
                .lock()
                .unwrap()
                .challenges
                .get_instructions(self.selected_challenge.clone())
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
                    for challenge in self.app_state.lock().unwrap().challenges.items.iter() {
                        ui.radio_value(
                            &mut self.selected_challenge,
                            challenge.command.clone(),
                            &challenge.command,
                        );
                    }
                    ui.separator();
                    if ui.button("Refresh").clicked() {
                        self.app_state.lock().unwrap().update_activity_timer();
                        self.active_challenge = None;
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
