use std::fmt::{self, Display, Formatter};

#[derive(PartialEq, Clone, Copy, serde::Deserialize, serde::Serialize)]
enum FilterOption {
    All,
    UniquePlayers,
    UniqueLanguage,
}

#[derive(Default, PartialEq, Copy, Clone, serde::Deserialize, serde::Serialize)]
enum Challenges {
    #[default]
    C2331,
    C2332,
    C2333,
}

impl Display for Challenges {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Challenges::C2331 => write!(f, "2331"),
            Challenges::C2332 => write!(f, "2332"),
            Challenges::C2333 => write!(f, "2333"),
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct ScoreBoardApp {
    challenge: Challenges,
    filter: FilterOption,
    sort_column: String,
}

impl Default for ScoreBoardApp {
    fn default() -> Self {
        Self {
            challenge: Challenges::default(),
            filter: FilterOption::All,
            sort_column: "time_ns".to_string(),
        }
    }
}

impl super::App for ScoreBoardApp {
    fn name(&self) -> &'static str {
        "☰ Score Board"
    }

    fn show(&mut self, ctx: &egui::Context, open: &mut bool) {
        egui::Window::new(self.name())
            .open(open)
            .default_width(400.0)
            .show(ctx, |ui| {
                use super::View as _;
                self.ui(ui);
            });
    }
}

const NUM_MANUAL_ROWS: usize = 20;

impl super::View for ScoreBoardApp {
    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            egui::ComboBox::from_label("Challenge")
                .selected_text(format!("{}", self.challenge))
                .show_ui(ui, |ui| {
                    ui.style_mut().wrap = Some(false);
                    ui.set_min_width(60.0);
                    ui.selectable_value(&mut self.challenge, Challenges::C2331, "2331");
                    ui.selectable_value(&mut self.challenge, Challenges::C2332, "2332");
                    ui.selectable_value(&mut self.challenge, Challenges::C2333, "2333");
                });

            ui.label("Filter:");
            ui.radio_value(&mut self.filter, FilterOption::All, "All");
            ui.radio_value(
                &mut self.filter,
                FilterOption::UniquePlayers,
                "Unique Players",
            );
            ui.radio_value(
                &mut self.filter,
                FilterOption::UniqueLanguage,
                "Unique Langauges",
            );

            {
                let max_rows = 100;
            }
        });

        ui.separator();

        // Leave room for the source code link after the table demo:
        use egui_extras::{Size, StripBuilder};
        StripBuilder::new(ui)
            .size(Size::remainder().at_least(100.0)) // for the table
            .size(Size::exact(10.5)) // for the source code link
            .vertical(|mut strip| {
                strip.cell(|ui| {
                    egui::ScrollArea::horizontal().show(ui, |ui| {
                        self.table_ui(ui);
                    });
                });
                strip.cell(|ui| {
                    ui.vertical_centered(|ui| {});
                });
            });
    }
}

impl ScoreBoardApp {
    fn table_ui(&mut self, ui: &mut egui::Ui) {
        use egui_extras::{Column, TableBuilder};

        let text_height = egui::TextStyle::Body.resolve(ui.style()).size;

        let mut table = TableBuilder::new(ui)
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
            .column(Column::auto())
            .column(Column::initial(100.0).range(40.0..=300.0))
            .column(Column::initial(100.0).at_least(40.0).clip(true))
            .column(Column::remainder())
            .min_scrolled_height(0.0);

        table
            .header(20.0, |mut header| {
                header.col(|ui| {
                    ui.strong("Row");
                });
                header.col(|ui| {
                    ui.strong("Expanding content");
                });
                header.col(|ui| {
                    ui.strong("Clipped text");
                });
                header.col(|ui| {
                    ui.strong("Content");
                });
            })
            .body(|mut body| {
                for row_index in 0..NUM_MANUAL_ROWS {
                    let is_thick = thick_row(row_index);
                    let row_height = if is_thick { 30.0 } else { 18.0 };
                    body.row(row_height, |mut row| {
                        row.col(|ui| {
                            ui.label(row_index.to_string());
                        });
                        row.col(|ui| {
                            expanding_content(ui);
                        });
                        row.col(|ui| {
                            ui.label(long_text(row_index));
                        });
                        row.col(|ui| {
                            ui.style_mut().wrap = Some(false);
                            if is_thick {
                                ui.heading("Extra thick row");
                            } else {
                                ui.label("Normal row");
                            }
                        });
                    });
                }
            });
    }
}

fn expanding_content(ui: &mut egui::Ui) {
    let width = ui.available_width().clamp(20.0, 200.0);
    let height = ui.available_height();
    let (rect, _response) = ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::hover());
    ui.painter().hline(
        rect.x_range(),
        rect.center().y,
        (1.0, ui.visuals().text_color()),
    );
}

fn long_text(row_index: usize) -> String {
    format!("Row {row_index} has some long text that you may want to clip, or it will take up too much horizontal space!")
}

fn thick_row(row_index: usize) -> bool {
    row_index % 6 == 0
}
