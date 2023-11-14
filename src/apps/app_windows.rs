use super::App;
use crate::helpers::AppState;
use egui::{Context, ScrollArea, Ui};
use std::collections::BTreeSet;
use std::sync::{Arc, Mutex};

// ----------------------------------------------------------------------------

#[derive(serde::Deserialize, serde::Serialize)]
struct Apps {
    #[serde(skip)]
    apps: Vec<Box<dyn App>>,
    open: BTreeSet<String>,
}

impl Default for Apps {
    fn default() -> Self {
        Self::from_apps(vec![
            Box::<super::login_app::LoginApp>::default(),
            Box::<super::scoreboard_app::ScoreBoardApp>::default(),
            Box::<super::challenge_info::ChallengeInfoApp>::default(),
            Box::<super::code_editor::CodeEditor>::default(),
            Box::<super::binary_upload::BinaryUpload>::default(),
            Box::<super::PasswordResetApp>::default(),
        ])
    }
}

impl Apps {
    pub fn from_apps(apps: Vec<Box<dyn App>>) -> Self {
        let mut open = BTreeSet::new();
        open.insert(super::login_app::LoginApp::default().name().to_owned());

        Self { apps, open }
    }

    #[allow(dead_code)] //inhibit warnings when target =/= WASM
    pub fn checkboxes(&mut self, ui: &mut Ui) {
        let Self { apps, open } = self;
        ui.label(format!("{} apps", apps.len()));
        for app in apps {
            let mut is_open = open.contains(app.name());
            ui.toggle_value(&mut is_open, app.name());
            set_open(open, app.name(), is_open);
        }
    }

    #[allow(dead_code)] //inhibit warnings when target =/= WASM
    pub fn windows(&mut self, ctx: &Context) {
        let Self { apps, open } = self;
        for app in apps {
            let mut is_open = open.contains(app.name());
            app.show(ctx, &mut is_open);
            set_open(open, app.name(), is_open);
        }
    }
}

// ----------------------------------------------------------------------------

fn set_open(open: &mut BTreeSet<String>, key: &'static str, is_open: bool) {
    if is_open {
        if !open.contains(key) {
            open.insert(key.to_owned());
        }
    } else {
        open.remove(key);
    }
}

// ----------------------------------------------------------------------------

#[derive(serde::Deserialize, serde::Serialize)]
pub struct AppWindows {
    about_is_open: bool,
    apps: Apps,
    #[serde(skip)]
    pub app_state: Arc<Mutex<AppState>>,
}

impl Default for AppWindows {
    fn default() -> Self {
        Self {
            about_is_open: true,
            apps: Default::default(),
            app_state: Arc::new(Mutex::new(AppState::default())),
        }
    }
}

impl AppWindows {
    #[allow(dead_code)] //inhibit warnings when target =/= WASM
    pub fn set_app_state_ref(&mut self, app_state: Arc<Mutex<AppState>>) {
        self.app_state = app_state;

        for app in &mut self.apps.apps {
            app.set_app_state_ref(Arc::clone(&self.app_state));
        }
    }
}

impl AppWindows {
    #[allow(dead_code)] //inhibit warnings when target =/= WASM
    pub fn ui(&mut self, ctx: &Context) {
        self.desktop_ui(ctx);
    }

    #[allow(dead_code)] //inhibit warnings when target =/= WASM
    fn desktop_ui(&mut self, ctx: &Context) {
        egui::SidePanel::right("egui_app_panel")
            .resizable(true)
            .default_width(300.0)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading("🎯 apps");
                });

                ui.separator();

                self.app_list_ui(ui);

                ui.separator();

                use egui::special_emojis::GITHUB;
                ui.hyperlink_to(
                    format!("{GITHUB} GitHub"),
                    "https://github.com/bitbrain-za/judge_2331-rs",
                );
                ui.add(
                    egui::Hyperlink::from_label_and_url(
                        "🐛 Report a UI bug",
                        "https://github.com/bitbrain-za/challenge_frontend/issues/new",
                    )
                    .open_in_new_tab(true),
                );
                ui.add(
                    egui::Hyperlink::from_label_and_url(
                        "🐛 Report a backend bug",
                        "https://github.com/bitbrain-za/challenge_backend/issues/new",
                    )
                    .open_in_new_tab(true),
                );
                ui.separator();
            });
        self.show_windows(ctx);
    }

    #[allow(dead_code)] //inhibit warnings when target =/= WASM
    fn show_windows(&mut self, ctx: &Context) {
        self.apps.windows(ctx);
    }

    #[allow(dead_code)] //inhibit warnings when target =/= WASM
    fn app_list_ui(&mut self, ui: &mut egui::Ui) {
        ScrollArea::vertical().show(ui, |ui| {
            ui.with_layout(egui::Layout::top_down_justified(egui::Align::LEFT), |ui| {
                // ui.toggle_value(&mut self.about_is_open, self.about.name());

                self.apps.checkboxes(ui);
            });
        });
    }
}
