use aider::{MessageStream, agent::openai::CodeAgent};
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    style::Stylize,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use dotenvy::dotenv;
use ratatui::{
    DefaultTerminal, Frame, Terminal,
    backend::CrosstermBackend,
    buffer::Buffer,
    layout::{self, Alignment, Constraint, Layout, Rect},
    style::{Color, Style},
    text::{self, Line, Masked, Span},
    widgets::{self, Block, Borders, List, ListItem, Paragraph, ScrollbarState, Widget, Wrap},
};
use std::{
    io,
    time::{Duration, Instant},
};
use tokio::sync::mpsc::{self, Receiver};

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
    let (tx, rx) = mpsc::channel::<MessageStream>(32);

    let terminal = ratatui::init();
    let app_result = App::new(rx, tx).run(terminal).await;
    ratatui::restore();
    app_result
}
#[derive(Debug)]
struct App {
    should_exit: bool,
    scroll: u16,
    last_tick: Instant,
    rx: mpsc::Receiver<MessageStream>,
    tx: mpsc::Sender<MessageStream>,

    state: AppState,
}

impl App {
    /// The duration between each tick.
    // const TICK_RATE: Duration = Duration::from_millis(250);
    const TICK_RATE: Duration = Duration::from_millis(10);

    /// Create a new instance of the app.
    fn new(rx: mpsc::Receiver<MessageStream>, tx: mpsc::Sender<MessageStream>) -> Self {
        Self {
            should_exit: false,
            scroll: 0,
            last_tick: Instant::now(),
            rx,
            tx,
            state: AppState::new(),
        }
    }

    /// Run the app until the user exits.
    async fn run(
        mut self,
        mut terminal: DefaultTerminal,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // async fn main() -> Result<(), Box<dyn std::error::Error>> {
        while !self.should_exit {
            terminal.draw(|frame| frame.render_widget(&self, frame.area()))?;
            self.handle_events()?;
            if self.last_tick.elapsed() >= Self::TICK_RATE {
                self.on_tick();
                self.last_tick = Instant::now();
            }
        }
        Ok(())
    }

    /// Handle events from the terminal.
    fn handle_events(&mut self) -> io::Result<()> {
        let agent = CodeAgent::new_from_env();

        if let Ok(message_stream) = self.rx.try_recv() {
            match message_stream {
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
            // self.state.messages.push(format!("AI: {}", chunk));
            // // Append streamed characters
            // if let Some(last) = self.state.messages.last_mut() {
            //     last.push_str(&chunk); // append text piece-by-piece
            // }
        }

        // let timeout = Self::TICK_RATE.saturating_sub(self.last_tick.elapsed());
        // while event::poll(timeout)? {
        while event::poll(Self::TICK_RATE)? {
            if let event::Event::Key(key) = event::read()? {
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
                            let tx_clone = self.tx.clone();
                            let agent_clone = agent.clone();
                            tokio::spawn(async move {
                                agent_clone.stream_ai_response(&user_msg, tx_clone).await
                                // match agent_clone.send(&user_msg).await {
                                //     Ok(resp) => {
                                //         let _ = tx_clone.send(resp).await;
                                //     }
                                //     Err(e) => {
                                //         let _ = tx_clone.send(format!("Error: {:?}", e)).await;
                                //     }
                                // }
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
        }
        Ok(())
    }

    /// Update the app state on each tick.
    fn on_tick(&mut self) {
        self.scroll = (self.scroll + 1) % 10;
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
