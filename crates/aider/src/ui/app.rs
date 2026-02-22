use super::component::Component;
use crate::agent::openai::CodeAgent;
use crate::channel::event_channel::EventChannel;
use crate::event::message_stream_event::MessageStreamEvent;
use crate::state::chat_session::ChatSession;
use crate::ui::chat_history::ChatHistory;
use crate::ui::chat_input::ChatInput;
use crate::ui::header::Header;
use crossterm::event::{self, Event, EventStream};
use futures_util::StreamExt;
use ratatui::prelude::Backend;
use ratatui::{Frame, Terminal};
use ratatui::{
    buffer::Buffer,
    layout::{self, Rect},
    widgets::Widget,
};
use std::io;
use tokio::select;
#[derive(Debug)]
pub struct App {
    should_exit: bool,
    /// input event channel
    ie_chan: EventChannel<Event>,
    /// message stream event channel
    mse_chan: EventChannel<MessageStreamEvent>,
    chat_session: ChatSession,
}
impl App {
    /// Create a new instance of the app.
    pub fn new() -> Self {
        let ie_chan = EventChannel::new();
        let mse_chan = EventChannel::new();
        Self {
            should_exit: false,
            ie_chan,
            mse_chan,
            chat_session: ChatSession::new(),
        }
    }
    fn render(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }
    fn redraw(&self, terminal: &mut Terminal<impl Backend>) -> io::Result<()> {
        let _ = terminal.draw(|frame| self.render(frame));
        Ok(())
    }
    /// listen input event from other thread
    /// and collect them in main thread
    fn listen_input_events(&mut self) {
        let tx = self.ie_chan.tx.clone();
        tokio::spawn(async move {
            let mut events = EventStream::new();
            while let Some(Ok(ev)) = events.next().await {
                tx.send(ev).await.unwrap();
            }
        });
    }
    /// Run the app until the user exits.
    pub async fn run(
        mut self,
        mut terminal: Terminal<impl Backend>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.redraw(&mut terminal)?;
        self.listen_input_events();
        // This loop waits for events.
        loop {
            select! {
                Some(input_event) = self.ie_chan.rx.recv() => {
                    self.handle_input_event(input_event)?;
                    self.redraw(&mut terminal)?;
                    if self.should_exit {
                        break;
                    }
                }
                Some(mse) = self.mse_chan.rx.recv() => {
                    match mse {
                        MessageStreamEvent::Start => {
                            self.chat_session.history.push("AI: ".to_string());
                        }
                        MessageStreamEvent::NextWord { word } => {
                            if let Some(last) = self.chat_session.history.last_mut() {
                                last.push_str(&word);
                            }
                        }
                        MessageStreamEvent::Completed => {}
                        MessageStreamEvent::Error { error } => {
                            self.chat_session.history.push(format!("Error: {}", error));
                        }
                    }
                    self.redraw(&mut terminal)?;
                }
                 else => break,
            }
        }
        Ok(())
    }

    fn handle_input_event(&mut self, input_event: Event) -> io::Result<()> {
        use event::Event::*;
        use event::KeyCode::*;

        if let Key(key) = input_event {
            match (key.code, key.modifiers) {
                (Char('c'), event::KeyModifiers::CONTROL) => {
                    self.should_exit = true;
                }
                (Char(c), _) => {
                    self.chat_session.draft.push(c);
                }
                (Backspace, _) => {
                    self.chat_session.draft.pop();
                }
                (Enter, _) => {
                    let user_msg = self.chat_session.draft.trim().to_string();
                    if !user_msg.is_empty() {
                        self.chat_session
                            .history
                            .push(format!("You: {}", self.chat_session.draft));
                        let tx = self.mse_chan.tx.clone();
                        tokio::spawn(async move {
                            let agent = CodeAgent::new_from_env();
                            agent.stream_ai_response(&user_msg, tx).await
                        });
                    }
                    self.chat_session.draft.clear();
                }
                (Esc, _) => {
                    self.should_exit = true;
                }
                _ => {}
            }
        }
        Ok(())
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        use layout::Constraint::*;
        use layout::Direction::*;

        let layout = layout::Layout::default()
            .direction(Vertical)
            .constraints([Length(3), Min(1), Length(3)])
            .split(area);

        Header.render(layout[0], buf);

        ChatHistory {
            history: self.chat_session.history.clone(),
        }
        .render(layout[1], buf);
        ChatInput {
            draft: self.chat_session.draft.clone(),
        }
        .render(layout[2], buf);
    }
}
