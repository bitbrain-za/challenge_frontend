use crate::helpers::{
    fetchers::{GetStatus, Requestor},
    Challenges,
};
use scoreboard_db::Builder as FilterBuilder;
use scoreboard_db::Filter as ScoreBoardFilter;
use scoreboard_db::{NiceTime, Score, ScoreBoard, SortColumn};
use std::{borrow::BorrowMut, str::FromStr};

#[derive(PartialEq, Clone, Copy, serde::Deserialize, serde::Serialize)]
enum FilterOption {
    All,
    UniquePlayers,
    UniqueLanguage,
}

#[derive(Clone, serde::Deserialize, serde::Serialize)]
enum FetchResponse {
    Success(Vec<Score>),
    Failure(String),
    FailAuth,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct ScoreBoardApp {
    challenge: Challenges,
    filter: FilterOption,
    sort_column: String,

    active_challenge: Challenges,
    active_filter: FilterOption,
    active_sort_column: String,

    scores: Option<Vec<Score>>,
    #[serde(skip)]
    url: String,

    #[serde(skip)]
    score_fetcher: Option<Requestor>,
}

impl Default for ScoreBoardApp {
    fn default() -> Self {
        Self {
            challenge: Challenges::default(),
            filter: FilterOption::All,
            sort_column: "time".to_string(),
            url: option_env!("BACKEND_URL")
                .unwrap_or("http://123.4.5.6:3000/")
                .to_string(),

            active_challenge: Challenges::None,
            active_filter: FilterOption::All,
            active_sort_column: "time".to_string(),
            scores: None,
            score_fetcher: None,
        }
    }
}

impl ScoreBoardApp {
    fn fetch(&mut self) {
        self.scores = None;

        let url = format!("{}api/game/scores/{}", self.url, self.challenge);

        log::debug!("Fetching challenge info");
        let mut getter = Requestor::new_get(&url, true);
        getter.send();
        self.score_fetcher = Some(getter);
    }

    fn check_for_reload(&mut self) -> bool {
        if self.active_challenge != self.challenge
            || self.active_filter != self.filter
            || self.active_sort_column != self.sort_column
        {
            self.active_challenge = self.challenge;
            self.active_filter = self.filter;
            self.active_sort_column = self.sort_column.clone();
            return true;
        }
        false
    }

    fn check_fetch_promise(&mut self) -> GetStatus {
        let getter = &mut self.score_fetcher;

        if let Some(getter) = getter {
            let result = &getter.check_promise();

            match result {
                GetStatus::Success(_) => {
                    self.score_fetcher = None;
                }
                GetStatus::Failed(_) => {
                    self.score_fetcher = None;
                }
                _ => {}
            }
            return result.clone();
        }
        GetStatus::NotStarted
    }
}

impl super::App for ScoreBoardApp {
    fn name(&self) -> &'static str {
        "☰ Score Board"
    }

    fn show(&mut self, ctx: &egui::Context, open: &mut bool) {
        if self.check_for_reload() {
            self.fetch();
        }

        if let Some(fetcher) = self.score_fetcher.borrow_mut() {
            if fetcher.refresh_context() {
                log::debug!("Refreshing context");
                ctx.request_repaint();
            }
        }

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
                        self.fetch();
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

        match self.check_fetch_promise() {
            GetStatus::Success(text) => {
                self.score_fetcher = None;
                self.scores = Some(serde_json::from_str(&text).unwrap());
            }
            GetStatus::Failed(e) => {
                self.score_fetcher = None;
                let message = format!("Failed to fetch scores: {}", e);
                log::error!("{}", message);
                ui.label(message);
            }
            GetStatus::InProgress => {
                ui.label("Fetching scores...");
            }
            GetStatus::NotStarted => {}
        }

        let text_height = egui::TextStyle::Body.resolve(ui.style()).size;
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
                if let Some(scores) = &self.scores {
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
