use crate::app;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use std::sync::mpsc::{self, Receiver, Sender};

pub enum SearchEvent {
    Term(String),
    Represent,
    RequestClose,
}

pub enum ManagerEvent {
    DisplayEntries(Vec<app::AppEntry>),
    Mathematic(f64),
    Clear,
    Close,
}

pub struct SearchManager {
    rx: Receiver<SearchEvent>,
    outsender: async_channel::Sender<ManagerEvent>,
    matcher: SkimMatcherV2,
    entries: Vec<app::AppEntry>,
}

impl SearchManager {
    pub fn new() -> (
        Self,
        (Sender<SearchEvent>, async_channel::Receiver<ManagerEvent>),
    ) {
        let (insender, rx) = mpsc::channel::<SearchEvent>();
        let (outsender, outrx) = async_channel::bounded::<ManagerEvent>(1);
        (
            Self {
                rx,
                outsender,
                matcher: SkimMatcherV2::default(),
                entries: app::collect_apps(),
            },
            (insender, outrx),
        )
    }

    pub fn manage(mut self) {
        tokio::spawn(async move {
            while let Ok(ev) = self.rx.recv() {
                match ev {
                    SearchEvent::Term(query) => {
                        let _ = self.outsender.send(ManagerEvent::Clear).await;
                        let entry_results: Vec<app::AppEntry> = self
                            .entries
                            .iter()
                            .filter_map(|entry| {
                                let entry = entry.clone();
                                let score =
                                    self.matcher.fuzzy_match(&entry.name, &query).unwrap_or(0);

                                if score > 0 {
                                    Some(entry)
                                } else {
                                    None
                                }
                            })
                            .collect();
                        if let Ok(res) = exmex::eval_str::<f64>(&query) {
                            let _ = self.outsender.send(ManagerEvent::Mathematic(res)).await;

                            let top_1 = &entry_results[..1.min(entry_results.len())];
                            if !top_1.is_empty() {
                                let _ = self
                                    .outsender
                                    .send(ManagerEvent::DisplayEntries(top_1.to_vec()))
                                    .await;
                            }
                        } else {
                            let top_5 = &entry_results[..10.min(entry_results.len())];
                            if !top_5.is_empty() {
                                let _ = self
                                    .outsender
                                    .send(ManagerEvent::DisplayEntries(top_5.to_vec()))
                                    .await;
                            }
                        }
                    }
                    SearchEvent::Represent => self.entries = app::collect_apps(),
                    SearchEvent::RequestClose => {
                        let _ = self.outsender.send(ManagerEvent::Close).await;
                    }
                }
            }
        });
    }
}
