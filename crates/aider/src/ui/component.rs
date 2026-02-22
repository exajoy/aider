use ratatui::prelude::*;

pub trait Component {
    fn render(self, area: Rect, buf: &mut Buffer);
}
