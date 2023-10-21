use crate::helpers::Challenges;
use egui_commonmark::*;
use poll_promise::Promise;

#[derive(PartialEq, Clone, Copy, serde::Deserialize, serde::Serialize)]
enum FilterOption {
    All,
    UniquePlayers,
    UniqueLanguage,
}

struct Resource {
    _response: ehttp::Response,
    text: Option<String>,
}

impl Resource {
    fn from_response(_: &egui::Context, response: ehttp::Response) -> Self {
        let _ = response.content_type().unwrap_or_default();
        let text = response.text();
        let text = text.map(|text| text.to_owned());

        Self {
            _response: response,
            text,
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct ChallengeInfoApp {
    challenge: Challenges,
    #[serde(skip)]
    promise: Option<Promise<ehttp::Result<Resource>>>,
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
        let url = self.challenge.get_info_url();
        let ctx = ctx.clone();
        let (sender, promise) = Promise::new();
        let request = ehttp::Request::get(url);
        ehttp::fetch(request, move |response| {
            ctx.request_repaint(); // wake up UI thread
            let resource = response.map(|response| Resource::from_response(&ctx, response));
            sender.send(resource);
        });
        self.promise = Some(promise);
        self.refresh = false;
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
            .default_width(400.0)
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
                                Ok(resource) => {
                                    let mut cache = CommonMarkCache::default();
                                    CommonMarkViewer::new("viewer").show(
                                        ui,
                                        &mut cache,
                                        resource.text.as_ref().unwrap(),
                                    );
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
