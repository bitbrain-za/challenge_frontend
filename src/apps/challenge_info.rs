use crate::helpers::Challenges;
use egui_commonmark::*;
use gloo_net::http;
use poll_promise::Promise;

#[derive(PartialEq, Clone, Copy, serde::Deserialize, serde::Serialize)]
enum FilterOption {
    All,
    UniquePlayers,
    UniqueLanguage,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct ChallengeInfoApp {
    challenge: Challenges,
    #[serde(skip)]
    promise: Option<Promise<Result<String, String>>>,
    #[serde(skip)]
    active_challenge: Challenges,
    #[serde(skip)]
    refresh: bool,
}

impl Default for ChallengeInfoApp {
    fn default() -> Self {
        Self {
            challenge: Challenges::default(),
            promise: Default::default(),
            refresh: true,

            active_challenge: Challenges::default(),
        }
    }
}

impl ChallengeInfoApp {
    fn fetch(&mut self, ctx: &egui::Context) {
        if !self.refresh {
            return;
        }
        self.refresh = false;

        let url = self.challenge.get_info_url();
        let ctx = ctx.clone();

        let promise = poll_promise::Promise::spawn_local(async move {
            let response = http::Request::get(&url);
            let response = response.send().await.unwrap();
            let text = response.text().await.map_err(|e| format!("{:?}", e));
            ctx.request_repaint(); // wake up UI thread
            text
        });
        self.promise = Some(promise);
    }

    fn check_for_reload(&mut self) {
        if self.active_challenge != self.challenge {
            self.active_challenge = self.challenge;
            self.refresh = true;
        }
    }
}

impl super::App for ChallengeInfoApp {
    fn name(&self) -> &'static str {
        "📖 Challenge Info"
    }

    fn show(&mut self, ctx: &egui::Context, open: &mut bool) {
        self.fetch(ctx);
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
        self.check_for_reload();

        egui::SidePanel::right("ChallengeInfoSelection")
            .resizable(false)
            .show_inside(ui, |ui| {
                ui.vertical(|ui| {
                    for challenge in Challenges::iter() {
                        ui.radio_value(&mut self.challenge, challenge, format!("{}", challenge));
                    }
                    ui.separator();
                    if ui.button("Refresh").clicked() {
                        self.refresh = true;
                    }
                });
            });
        egui::CentralPanel::default().show_inside(ui, |ui| {
            egui::ScrollArea::both()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    if let Some(promise) = &self.promise {
                        if let Some(result) = promise.ready() {
                            match result {
                                Ok(text) => {
                                    let mut cache = CommonMarkCache::default();
                                    CommonMarkViewer::new("viewer").show(ui, &mut cache, text);
                                }
                                Err(err) => {
                                    ui.label(format!("Error fetching file: {}", err));
                                }
                            }
                        }
                    }
                });
        });
    }
}
