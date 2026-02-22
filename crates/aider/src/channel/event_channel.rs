use tokio::sync::mpsc;

#[derive(Debug)]
pub struct EventChannel<T> {
    pub tx: mpsc::Sender<T>,
    pub rx: mpsc::Receiver<T>,
}
impl<T> EventChannel<T> {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel::<T>(32);
        Self { tx, rx }
    }
}
