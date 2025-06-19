use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use std::time::Duration;

#[derive(Debug, Clone)]
pub enum AppEvent {
    Tick,
    Quit,
    Recalc,
    CharInput(char),
    Backspace,
    Enter,
    NextRound,
    ToggleTarget,
}

pub struct EventHandler {
    // No need to store anything, just handle events
}

impl EventHandler {
    pub fn new() -> Self {
        Self {}
    }

    pub fn next(&self) -> Result<AppEvent, Box<dyn std::error::Error>> {
        // Check for key events with a short timeout
        if event::poll(Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                    match key_event.code {
                        KeyCode::Char('q') | KeyCode::Esc => Ok(AppEvent::Quit),
                        KeyCode::Char('r') => Ok(AppEvent::Recalc),
                        KeyCode::Char('n') => Ok(AppEvent::NextRound),
                        KeyCode::Char('t') => Ok(AppEvent::ToggleTarget),
                        KeyCode::Char(c) => Ok(AppEvent::CharInput(c)),
                        KeyCode::Backspace => Ok(AppEvent::Backspace),
                        KeyCode::Enter => Ok(AppEvent::Enter),
                        _ => Ok(AppEvent::Tick),
                    }
                }
                _ => Ok(AppEvent::Tick),
            }
        } else {
            Ok(AppEvent::Tick)
        }
    }
}
