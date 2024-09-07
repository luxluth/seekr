use std::{
    sync::mpsc::{self, Receiver, Sender},
    thread,
};

pub enum SearchEvent {
    Term(String),
}

pub enum ManagerEvent {}

pub struct SearchManager {
    rx: Receiver<SearchEvent>,
    outsender: Sender<ManagerEvent>,
}

impl SearchManager {
    pub fn new() -> (Self, (Sender<SearchEvent>, Receiver<ManagerEvent>)) {
        let (insender, rx) = mpsc::channel::<SearchEvent>();
        let (outsender, outrx) = mpsc::channel::<ManagerEvent>();
        (Self { rx, outsender }, (insender, outrx))
    }

    pub fn manage(self) {
        thread::spawn(move || {
            while let Ok(ev) = self.rx.recv() {
                match ev {
                    SearchEvent::Term(e) => {
                        eprintln!("{e}");
                    }
                }
            }
        });
    }
}
