use crossterm::event::Event;

pub mod agent;

#[derive(Debug)]
pub enum MessageStream {
    Start,
    NextWord { word: String },
    Completed,
    Error { error: String },
}
#[derive(Debug)]
pub enum AppEvent {
    Input(Event),
    IncomingMessage(MessageStream),
    Redraw,
}
