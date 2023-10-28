use std::collections::BTreeSet;

use egui::{Context, ScrollArea, Ui};

use super::App;

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
            // Box::<super::login_app::LoginApp>::default(),
            Box::<super::scoreboard_app::ScoreBoardApp>::default(),
            // Box::<super::challenge_info::ChallengeInfoApp>::default(),
            // Box::<super::code_editor::CodeEditor>::default(),
        ])
    }
}

impl Apps {
    pub fn from_apps(apps: Vec<Box<dyn App>>) -> Self {
        let mut open = BTreeSet::new();
        // open.insert(super::login_app::LoginApp::default().name().to_owned());

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
}

impl Default for AppWindows {
    fn default() -> Self {
        Self {
            about_is_open: true,
            apps: Default::default(),
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
            .resizable(false)
            .default_width(150.0)
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
