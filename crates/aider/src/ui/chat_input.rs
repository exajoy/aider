use super::component::Component;
use ratatui::{
    layout::Alignment,
    prelude::{Buffer, Rect, Widget},
    style::{Color, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph},
};
pub struct ChatInput {
    pub draft: String,
}

impl Component for ChatInput {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let layout = Paragraph::new(self.draft.as_str())
            .style(Style::default().fg(Color::Yellow))
            .block(
                Block::default()
                    .title_top(Line::from("Enter to send").alignment(Alignment::Left))
                    .borders(Borders::ALL),
            );

        layout.render(area, buf);
    }
}
