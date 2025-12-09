use aider::AppEvent;
use aider::{MessageStream, agent::openai::CodeAgent};
use crossterm::event::{self, Event, EventStream};
use dotenvy::dotenv;
use futures_util::StreamExt;
use ratatui::{
    DefaultTerminal,
    buffer::Buffer,
    layout::{self, Rect},
    style::{Color, Style},
    text::{self},
    widgets::{Block, Borders, List, ListItem, Paragraph, Widget},
};
use std::io;
use tokio::sync::mpsc::{self};

#[derive(Debug)]
struct AppState {
    messages: Vec<String>,
    input: String,
}

impl AppState {
    fn new() -> Self {
        Self {
            messages: vec!["Welcome to AI Chat!".into()],
            input: String::new(),
        }
    }
}
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    color_eyre::install()?;

    dotenv().ok();

    let terminal = ratatui::init();
    let app_result = App::new().run(terminal).await;
    ratatui::restore();
    app_result
}
#[derive(Debug)]
struct App {
    should_exit: bool,
    event_rx: mpsc::Receiver<AppEvent>,
    event_tx: mpsc::Sender<AppEvent>,
    state: AppState,
}

impl App {
    /// Create a new instance of the app.
    fn new() -> Self {
        let (event_tx, event_rx) = mpsc::channel::<AppEvent>(32);
        Self {
            should_exit: false,
            event_rx,
            event_tx,
            state: AppState::new(),
        }
    }

    /// Run the app until the user exits.
    async fn run(
        mut self,
        mut terminal: DefaultTerminal,
    ) -> Result<(), Box<dyn std::error::Error>> {
        terminal.draw(|frame| frame.render_widget(&self, frame.area()))?;
        let tx = self.event_tx.clone();

        {
            tokio::spawn(async move {
                let mut events = EventStream::new();
                while let Some(Ok(ev)) = events.next().await {
                    tx.send(AppEvent::Input(ev)).await.unwrap();
                }
            });
        }
        // This loop never draws.
        // It only waits for events.
        let tx = self.event_tx.clone();
        loop {
            match self.event_rx.recv().await {
                Some(AppEvent::Input(ev)) => {
                    self.handle_events(ev)?;

                    tx.send(AppEvent::Redraw).await.unwrap();
                    if self.should_exit {
                        break;
                    }
                }
                Some(AppEvent::IncomingMessage(msg)) => {
                    match msg {
                        MessageStream::Start => {
                            self.state.messages.push("AI: ".to_string());
                        }
                        MessageStream::NextWord { word } => {
                            if let Some(last) = self.state.messages.last_mut() {
                                last.push_str(&word);
                            }
                        }
                        MessageStream::Completed => {}
                        MessageStream::Error { error } => {
                            self.state.messages.push(format!("Error: {}", error));
                        }
                    }
                    tx.send(AppEvent::Redraw).await.unwrap();
                }
                Some(AppEvent::Redraw) => {
                    terminal.draw(|frame| frame.render_widget(&self, frame.area()))?;
                }

                None => break,
            }
        }
        Ok(())
    }

    fn handle_events(&mut self, event: Event) -> io::Result<()> {
        let agent = CodeAgent::new_from_env();

        if let event::Event::Key(key) = event {
            match key.code {
                event::KeyCode::Char(c) => {
                    self.state.input.push(c);
                }
                event::KeyCode::Backspace => {
                    self.state.input.pop();
                }
                event::KeyCode::Enter => {
                    let user_msg = self.state.input.trim().to_string();
                    if !user_msg.is_empty() {
                        self.state
                            .messages
                            .push(format!("You: {}", self.state.input.clone()));
                        // Spawn the agent request
                        let tx_clone = self.event_tx.clone();
                        let agent_clone = agent.clone();
                        tokio::spawn(async move {
                            agent_clone.stream_ai_response(&user_msg, tx_clone).await
                        });
                    }
                    self.state.input.clear();
                }
                event::KeyCode::Esc => {
                    self.should_exit = true;
                }
                _ => {}
            }
            match (key.code, key.modifiers) {
                (event::KeyCode::Char('c'), event::KeyModifiers::CONTROL) => {
                    self.should_exit = true;
                }
                _ => {}
            }
        }
        // }
        Ok(())
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let chunks = layout::Layout::default()
            .direction(layout::Direction::Vertical)
            .constraints([layout::Constraint::Min(1), layout::Constraint::Length(3)])
            .split(area);

        let items: Vec<ListItem> = self
            .state
            .messages
            .iter()
            .map(|m| ListItem::new(m.clone()))
            .collect();

        let messages_list =
            List::new(items).block(Block::default().title("Messages").borders(Borders::ALL));

        messages_list.render(chunks[0], buf);

        let input_box = Paragraph::new(self.state.input.as_str())
            .style(Style::default().fg(Color::Yellow))
            .block(
                Block::default()
                    .title_top(text::Line::from("Enter to send").alignment(layout::Alignment::Left))
                    .borders(Borders::ALL),
            );

        input_box.render(chunks[1], buf);
    }
}
