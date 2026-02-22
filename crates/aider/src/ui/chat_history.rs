use super::component::Component;
use ratatui::{
    prelude::{Buffer, Rect, Widget},
    widgets::{Block, Borders, List, ListItem},
};
pub struct ChatHistory {
    pub history: Vec<String>,
}

impl Component for ChatHistory {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let items: Vec<ListItem> = self
            .history
            .iter()
            .map(|m| ListItem::new(m.clone()))
            .collect();

        let layout =
            List::new(items).block(Block::default().title("ChatHistory").borders(Borders::ALL));

        layout.render(area, buf);
    }
}
