use poll_promise::Promise;
use scoreboard_db::Builder as FilterBuilder;
use scoreboard_db::{NiceTime, Score, ScoreBoard};
use std::collections::HashMap;
use std::fmt::{self, Display, Formatter};
use std::sync::Mutex;
use std::time::Duration;

#[derive(PartialEq, Clone, Copy, serde::Deserialize, serde::Serialize)]
enum FilterOption {
    All,
    UniquePlayers,
    UniqueLanguage,
}

#[derive(Default, PartialEq, Eq, Hash, Copy, Clone, serde::Deserialize, serde::Serialize)]
enum Challenges {
    #[default]
    C2331,
    C2332,
    C2333,
}

struct Resource {
    response: ehttp::Response,
    scores: Option<String>,
}

impl Resource {
    fn from_response(ctx: &egui::Context, response: ehttp::Response) -> Self {
        let content_type = response.content_type().unwrap_or_default();
        let text = response.text();
        let text = text.map(|text| text.to_owned());

        log::debug!("Content-Type: {}", content_type);
        log::debug!("Text: {}", text.as_ref().map_or("None", String::as_str));

        Self {
            response,
            scores: text,
        }
    }
}

impl Challenges {
    fn next(&self) -> Self {
        match self {
            Challenges::C2331 => Challenges::C2332,
            Challenges::C2332 => Challenges::C2333,
            Challenges::C2333 => Challenges::C2331,
        }
    }
}

impl Display for Challenges {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Challenges::C2331 => write!(f, "23_3_1"),
            Challenges::C2332 => write!(f, "23_3_2"),
            Challenges::C2333 => write!(f, "23_3_3"),
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct ScoreBoardApp {
    challenge: Challenges,
    filter: FilterOption,
    sort_column: String,

    #[serde(skip)]
    promise: Option<Promise<ehttp::Result<Resource>>>,
    #[serde(skip)]
    url: String,
}

impl Default for ScoreBoardApp {
    fn default() -> Self {
        Self {
            challenge: Challenges::default(),
            filter: FilterOption::All,
            sort_column: "time_ns".to_string(),
            promise: Default::default(),
            url: "http://127.0.0.1:3000/scores/".to_string(),
        }
    }
}

impl ScoreBoardApp {
    fn fetch(&mut self, ctx: &egui::Context) {
        let url = format!("{}{}", self.url, self.challenge);
        let ctx = ctx.clone();
        let (sender, promise) = Promise::new();
        let request = ehttp::Request::get(url);
        ehttp::fetch(request, move |response| {
            ctx.request_repaint(); // wake up UI thread
            let resource = response.map(|response| Resource::from_response(&ctx, response));
            sender.send(resource);
        });
        self.promise = Some(promise);
    }
}

impl super::App for ScoreBoardApp {
    fn name(&self) -> &'static str {
        "☰ Score Board"
    }

    fn show(&mut self, ctx: &egui::Context, open: &mut bool) {
        self.fetch(ctx);
        egui::Window::new(self.name())
            .open(open)
            .default_width(400.0)
            .show(ctx, |ui| {
                use super::View as _;
                self.ui(ui);
            });
    }
}

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
        log::debug!("Table UI");

        let text_height = egui::TextStyle::Body.resolve(ui.style()).size;

        let mut table = TableBuilder::new(ui)
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
            .column(Column::auto())
            .column(Column::auto())
            .column(Column::initial(100.0).range(40.0..=300.0))
            .column(Column::initial(100.0).at_least(40.0).clip(true))
            .column(Column::remainder())
            .min_scrolled_height(0.0);

        table
            .header(20.0, |mut header| {
                header.col(|ui| {
                    ui.strong("#");
                });
                header.col(|ui| {
                    ui.strong("Time");
                });
                header.col(|ui| {
                    ui.strong("Name");
                });
                header.col(|ui| {
                    ui.strong("Language");
                });
                header.col(|ui| {
                    ui.strong("Binary");
                });
            })
            .body(|mut body| {
                // let scores = SCORES.lock().unwrap();
                // let scores = scores.get(&self.challenge).unwrap();

                // for (i, score) in scores.iter().enumerate() {
                //     let time = NiceTime::new(score.time_ns);
                //     let name = score.name.clone();
                //     let language = score.language.clone();
                //     let binary = score.command.clone();

                //     body.row(text_height, |mut row| {
                //         row.col(|ui| {
                //             ui.label(i.to_string());
                //         });
                //         row.col(|ui| {
                //             ui.label(time.to_string());
                //         });
                //         row.col(|ui| {
                //             ui.label(name);
                //         });
                //         row.col(|ui| {
                //             ui.label(language);
                //         });
                //         row.col(|ui| {
                //             ui.label(binary);
                //         });
                //     });
                // }
            });
    }
}
