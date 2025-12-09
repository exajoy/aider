pub mod agent;

#[derive(Debug)]
pub enum MessageStream {
    Start,
    NextWord { word: String },
    Completed,
    Error { error: String },
}
