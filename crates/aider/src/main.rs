use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};
use std::io;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut input = String::new();
    let mut messages: Vec<String> = vec![];

    loop {
        terminal.draw(|f| {
            let size = f.size();

            // Split: top = messages, bottom = input
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(1), Constraint::Length(3)])
                .split(size);

            // Render message list
            let items: Vec<ListItem> = messages.iter().map(|m| ListItem::new(m.clone())).collect();

            let messages_list =
                List::new(items).block(Block::default().title("Messages").borders(Borders::ALL));

            f.render_widget(messages_list, chunks[0]);

            // Render input box
            let input_box = Paragraph::new(input.as_str())
                .style(Style::default().fg(Color::Yellow))
                .block(Block::default().title("Input").borders(Borders::ALL));

            f.render_widget(input_box, chunks[1]);
        })?;

        // Handle key events
        if event::poll(std::time::Duration::from_millis(10))? {
            if let Event::Key(k) = event::read()? {
                match k.code {
                    KeyCode::Char(c) => {
                        input.push(c);
                    }
                    KeyCode::Backspace => {
                        input.pop();
                    }
                    KeyCode::Enter => {
                        if !input.trim().is_empty() {
                            messages.push(input.clone());
                        }
                        input.clear();
                    }
                    KeyCode::Esc => {
                        break;
                    }
                    _ => {}
                }
            }
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}
