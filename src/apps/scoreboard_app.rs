use crate::helpers::Challenges;
use poll_promise::Promise;
use scoreboard_db::Builder as FilterBuilder;
use scoreboard_db::Filter as ScoreBoardFilter;
use scoreboard_db::{NiceTime, Score, ScoreBoard, SortColumn};
use std::str::FromStr;

#[derive(PartialEq, Clone, Copy, serde::Deserialize, serde::Serialize)]
enum FilterOption {
    All,
    UniquePlayers,
    UniqueLanguage,
}

struct Resource {
    _response: ehttp::Response,
    scores: Vec<Score>,
}

impl Resource {
    fn from_response(_: &egui::Context, response: ehttp::Response) -> Self {
        let _ = response.content_type().unwrap_or_default();
        let text = response.text();
        let text = text.map(|text| text.to_owned());
        let scores: Vec<Score> = serde_json::from_str(text.as_ref().unwrap()).unwrap();

        Self {
            _response: response,
            scores,
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct ScoreBoardApp {
    challenge: Challenges,
    filter: FilterOption,
    sort_column: String,

    active_challenge: Challenges,
    active_filter: FilterOption,
    active_sort_column: String,

    #[serde(skip)]
    promise: Option<Promise<ehttp::Result<Resource>>>,
    #[serde(skip)]
    url: String,
    #[serde(skip)]
    refresh: bool,
}

impl Default for ScoreBoardApp {
    fn default() -> Self {
        Self {
            challenge: Challenges::default(),
            filter: FilterOption::All,
            sort_column: "time".to_string(),
            promise: Default::default(),
            url: option_env!("BACKEND_URL")
                .unwrap_or("http://123.4.5.6:3000/")
                .to_string(),
            refresh: true,

            active_challenge: Challenges::default(),
            active_filter: FilterOption::All,
            active_sort_column: "time".to_string(),
        }
    }
}

impl ScoreBoardApp {
    fn fetch(&mut self, ctx: &egui::Context) {
        if !self.refresh {
            return;
        }
        let url = format!("{}scores/{}", self.url, self.challenge);
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
        if self.active_challenge != self.challenge
            || self.active_filter != self.filter
            || self.active_sort_column != self.sort_column
        {
            self.active_challenge = self.challenge;
            self.active_filter = self.filter;
            self.active_sort_column = self.sort_column.clone();
            self.refresh = true;
        }
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

impl super::View for ScoreBoardApp {
    fn ui(&mut self, ui: &mut egui::Ui) {
        self.check_for_reload();

        egui::SidePanel::right("Options")
            .resizable(false)
            .show_inside(ui, |ui| {
                ui.vertical(|ui| {
                    egui::ComboBox::from_label("Challenge")
                        .selected_text(format!("{}", self.challenge))
                        .show_ui(ui, |ui| {
                            ui.style_mut().wrap = Some(false);
                            ui.set_min_width(60.0);

                            for challenge in Challenges::iter() {
                                ui.selectable_value(
                                    &mut self.challenge,
                                    challenge,
                                    format!("{}", challenge),
                                );
                            }
                        });

                    ui.separator();
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
                    self.table_ui(ui);
                });
        });
    }
}

impl ScoreBoardApp {
    fn table_ui(&mut self, ui: &mut egui::Ui) {
        use egui_extras::{Column, TableBuilder};

        let text_height = egui::TextStyle::Body.resolve(ui.style()).size;
        let mut scores: Option<Vec<Score>> = None;
        if let Some(promise) = &self.promise {
            if let Some(result) = promise.ready() {
                match result {
                    Ok(resource) => {
                        scores = Some(resource.scores.clone());
                    }
                    Err(error) => {
                        log::error!("Failed to fetch scores: {}", error);
                    }
                }
            }
        }

        let table = TableBuilder::new(ui)
            .striped(true)
            .resizable(true)
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
                if let Some(scores) = scores {
                    let mut filters = FilterBuilder::new();
                    match self.filter {
                        FilterOption::All => {}
                        FilterOption::UniquePlayers => {
                            filters.append(ScoreBoardFilter::UniquePlayers);
                        }
                        FilterOption::UniqueLanguage => {
                            filters.append(ScoreBoardFilter::UniquePlayers);
                        }
                    };

                    filters.append(ScoreBoardFilter::Sort(
                        SortColumn::from_str(self.sort_column.as_str()).expect("Invalid Cloumn"),
                    ));
                    let scores = ScoreBoard::new(scores.clone()).filter(filters).scores();

                    for (i, score) in scores.iter().enumerate() {
                        let time = NiceTime::new(score.time_ns);
                        let name = score.name.clone();
                        let language = score.language.clone();
                        let binary = score.command.clone();

                        body.row(text_height, |mut row| {
                            row.col(|ui| {
                                ui.label(i.to_string());
                            });
                            row.col(|ui| {
                                ui.label(time.to_string());
                            });
                            row.col(|ui| {
                                ui.label(name);
                            });
                            row.col(|ui| {
                                ui.label(language);
                            });
                            row.col(|ui| {
                                ui.label(binary);
                            });
                        });
                    }
                }
            });
    }
}
