use super::component::Component;
use ratatui::{
    prelude::{Buffer, Rect, Widget},
    widgets::{Block, Borders, Paragraph},
};
pub struct Header;

impl Component for Header {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let widget = Paragraph::new("CLI Chat")
            .block(Block::default().borders(Borders::ALL).title("Header"));
        widget.render(area, buf);
    }
}
