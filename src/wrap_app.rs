use crate::{
    apps::{self},
    background_processes::{ChallengeFetcher, LoginFetcher},
    code_editor,
    helpers::AppState,
};
#[cfg(target_arch = "wasm32")]
use core::any::Any;
use std::sync::{Arc, Mutex};

#[derive(serde::Deserialize, serde::Serialize, Default)]
struct CodeEditorApp {
    pub editor: code_editor::CodeEditor,
}

impl eframe::App for CodeEditorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.editor.panels(ctx);
    }
}

#[derive(serde::Deserialize, serde::Serialize, Default)]
#[serde(default)]
pub struct CodeChallengeApp {
    #[serde(skip)]
    pub windows: apps::app_windows::AppWindows,
}

impl eframe::App for CodeChallengeApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.windows.ui(ctx);

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                powered_by_egui_and_eframe(ui);
                egui::warn_if_debug_build(ui);
            });
        });
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize, Default)]
enum Anchor {
    #[default]
    Landing,
    CodeEditor,
}

impl Anchor {
    #[cfg(target_arch = "wasm32")]
    fn all() -> Vec<Self> {
        vec![Anchor::Landing, Anchor::CodeEditor]
    }
}

impl std::fmt::Display for Anchor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl From<Anchor> for egui::WidgetText {
    fn from(value: Anchor) -> Self {
        Self::RichText(egui::RichText::new(value.to_string()))
    }
}

#[derive(serde::Deserialize, serde::Serialize, Default)]
pub struct State {
    landing: CodeChallengeApp,
    code_editor: CodeEditorApp,
    selected_anchor: Anchor,
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct WrapApp {
    state: State,
    #[serde(skip)]
    app_state: Arc<Mutex<AppState>>,
    #[serde(skip)]
    challenge_fetcher: ChallengeFetcher,
    #[serde(skip)]
    login_fetcher: LoginFetcher,
}

impl Default for WrapApp {
    fn default() -> Self {
        let app_state = Arc::new(Mutex::new(AppState::default()));
        Self {
            state: State::default(),
            app_state,
            challenge_fetcher: ChallengeFetcher::default(),
            login_fetcher: LoginFetcher::default(),
        }
    }
}

impl WrapApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let app_state = AppState::default();
        let app_state = Arc::new(Mutex::new(app_state));

        let mut slf = Self {
            state: State::default(),
            app_state: Arc::clone(&app_state),
            challenge_fetcher: ChallengeFetcher::new(app_state.clone()),
            login_fetcher: LoginFetcher::new(app_state.clone()),
            #[cfg(any(feature = "glow", feature = "wgpu"))]
            custom3d: crate::apps::Custom3d::new(cc),
        };

        slf.state.code_editor.editor.app_state = Arc::clone(&app_state);
        slf.state
            .landing
            .windows
            .set_app_state_ref(Arc::clone(&app_state));

        if let Some(storage) = cc.storage {
            if let Some(state) = eframe::get_value(storage, eframe::APP_KEY) {
                slf.state = state;
            }
        }

        slf
    }

    fn apps_iter_mut(&mut self) -> impl Iterator<Item = (&str, Anchor, &mut dyn eframe::App)> {
        let vec = vec![
            (
                "✨ Landing",
                Anchor::Landing,
                &mut self.state.landing as &mut dyn eframe::App,
            ),
            (
                "💻 Code Editor",
                Anchor::CodeEditor,
                &mut self.state.code_editor as &mut dyn eframe::App,
            ),
        ];
        vec.into_iter()
    }
}

impl eframe::App for WrapApp {
    #[cfg(feature = "persistence")]
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, &self.state);
    }

    fn clear_color(&self, visuals: &egui::Visuals) -> [f32; 4] {
        visuals.panel_fill.to_normalized_gamma_f32()
    }

    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        #[cfg(target_arch = "wasm32")]
        if let Some(anchor) = frame.info().web_info.location.hash.strip_prefix('#') {
            let anchor = Anchor::all().into_iter().find(|x| x.to_string() == anchor);
            if let Some(v) = anchor {
                self.state.selected_anchor = v;
            }
        }

        self.challenge_fetcher.tick();
        self.login_fetcher.tick();

        #[cfg(not(target_arch = "wasm32"))]
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::F11)) {
            frame.set_fullscreen(!frame.info().window_info.fullscreen);
        }

        egui::TopBottomPanel::top("wrap_app_top_bar").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.visuals_mut().button_frame = false;
                self.bar_contents(ui, frame);
            });
        });

        self.show_selected_app(ctx, frame);

        // On web, the browser controls `pixels_per_point`.
        if !frame.is_web() {
            egui::gui_zoom::zoom_with_keyboard_shortcuts(ctx, frame.info().native_pixels_per_point);
        }
    }

    #[cfg(feature = "glow")]
    fn on_exit(&mut self, gl: Option<&glow::Context>) {
        if let Some(custom3d) = &mut self.custom3d {
            custom3d.on_exit(gl);
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn as_any_mut(&mut self) -> Option<&mut dyn Any> {
        Some(&mut *self)
    }
}

impl WrapApp {
    fn bar_contents(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        egui::widgets::global_dark_light_mode_switch(ui);

        ui.separator();

        let mut selected_anchor = self.state.selected_anchor;
        for (name, anchor, _app) in self.apps_iter_mut() {
            if ui
                .selectable_label(selected_anchor == anchor, name)
                .clicked()
            {
                selected_anchor = anchor;
                if frame.is_web() {
                    ui.ctx()
                        .open_url(egui::OpenUrl::same_tab(format!("#{anchor}")));
                }
            }
        }
        self.state.selected_anchor = selected_anchor;

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            egui::warn_if_debug_build(ui);
        });
    }

    fn show_selected_app(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let selected_anchor = self.state.selected_anchor;
        for (_name, anchor, app) in self.apps_iter_mut() {
            if anchor == selected_anchor || ctx.memory(|mem| mem.everything_is_visible()) {
                app.update(ctx, frame);
            }
        }
    }
}

fn powered_by_egui_and_eframe(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label("Powered by ");
        ui.hyperlink_to("egui", "https://github.com/emilk/egui");
        ui.label(" and ");
        ui.hyperlink_to(
            "eframe",
            "https://github.com/emilk/egui/tree/master/crates/eframe",
        );
        ui.label(".");
    });
}
