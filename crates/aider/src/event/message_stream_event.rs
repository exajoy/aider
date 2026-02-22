/// this is the message from server
/// which is use IncomingMessage event
/// in app event
#[derive(Debug)]
pub enum MessageStreamEvent {
    Start,
    NextWord { word: String },
    Completed,
    Error { error: String },
}
