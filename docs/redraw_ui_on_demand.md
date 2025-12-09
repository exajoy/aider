## Redraw UI on demand

A lot of Ratatui based TUIs run an infinite loop to
redraw the UI, even when it is not necessary.
[event-loop](https://ratatui.rs/templates/component/tui-rs/#event-loop)

Aider use a very clean approach, that
only draws the UI when there is:

- input event from user
- response event from AI model

This approach brings several benefits:

- saves CPU usage
- draw instantly
