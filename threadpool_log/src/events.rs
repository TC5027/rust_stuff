use std::time::Instant;

pub type Color = (u8, u8, u8);

#[derive(Debug)]
pub enum EventCategory {
    AddRequest,
    AddTasks(usize),
    StartProcessing,
    EndProcessing,
    Steal(usize),
}

#[derive(Debug)]
pub struct Event {
    pub category: EventCategory,
    pub time: Instant,
    pub color: Color,
}

pub type EventLog = Vec<Event>;

// on pourrait faire ça dynamique l'affichage svg avec wasm
// en overridant la méthode push pour eventlog, ça transmettrait
// à la page web pour l'affichage je sais pas
