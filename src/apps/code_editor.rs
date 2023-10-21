use crate::helpers::Languages;

#[derive(PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct CodeEditor {
    language: Languages,
    code: String,
}

impl Default for CodeEditor {
    fn default() -> Self {
        Self {
            language: Languages::Python,
            code: "#A very simple example\nprint(\"Hello world!\")".into(),
        }
    }
}

impl super::App for CodeEditor {
    fn name(&self) -> &'static str {
        "💻 Code Editor"
    }

    fn show(&mut self, ctx: &egui::Context, open: &mut bool) {
        use super::View as _;
        egui::Window::new(self.name())
            .open(open)
            .default_height(500.0)
            .show(ctx, |ui| self.ui(ui));
    }
}

impl super::View for CodeEditor {
    fn ui(&mut self, ui: &mut egui::Ui) {
        let Self { language, code } = self;

        ui.horizontal(|ui| {
            ui.set_height(0.0);
        });

        ui.horizontal(|ui| {
            ui.label("Language:");

            for l in Languages::iter() {
                ui.selectable_value(language, l, format!("{}", l));
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
                &language.to_string(),
            );
            layout_job.wrap.max_width = wrap_width;
            ui.fonts(|f| f.layout_job(layout_job))
        };

        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.add(
                egui::TextEdit::multiline(code)
                    .font(egui::TextStyle::Monospace) // for cursor height
                    .code_editor()
                    .desired_rows(10)
                    .lock_focus(true)
                    .desired_width(f32::INFINITY)
                    .layouter(&mut layouter),
            );
        });
    }
}
