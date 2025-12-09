## Redraw UI on demand

A lot of Ratatui based TUIs run an [infinite loop](https://ratatui.rs/templates/component/tui-rs/#event-loop) to
redraw the UI, even when it is not necessary.

Aider uses a very clean approach, that
only draws the UI when there is:

- input event from user
- message coming from AI model

This approach brings several benefits:

- significantly reduce CPU usage
- instant, responsive redraw
