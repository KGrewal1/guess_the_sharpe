mod app;
mod dist;
mod event;
mod ui;

use app::{App, AppMode, Guess};
use clap::Parser;
use compact_str::CompactString;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use event::{AppEvent, EventHandler};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io;

#[derive(Parser)]
#[command(name = "guess_the_sharpe")]
#[command(about = "A TUI application for visualizing and guessing Sharpe ratios")]
struct Cli {
    /// Enable guessing mode
    #[arg(short = 'g', long = "guess")]
    guessing_mode: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and run it
    let mode = if cli.guessing_mode {
        AppMode::Guessing(Guess {
            state: app::GuessState::WaitingForGuess,
            target: app::GuessTarget::Sample,
            current_guess: CompactString::with_capacity(12),
            score: 0,
            last_guess: None,
            guess_was_correct: false,
        })
    } else {
        AppMode::Display
    };
    let mut app = App::new(mode);
    let event_handler = EventHandler::new();
    let res = run_app(&mut terminal, &mut app, &event_handler);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    event_handler: &EventHandler,
) -> Result<(), Box<dyn std::error::Error>> {
    while app.running {
        terminal.draw(|f| ui::ui(f, app))?;

        match event_handler.next()? {
            AppEvent::Quit => app.quit(),
            AppEvent::Recalc => app.recalc(),
            AppEvent::CharInput(c) => app.add_char_to_guess(c),
            AppEvent::Backspace => app.remove_char_from_guess(),
            AppEvent::Enter => app.submit_guess(),
            AppEvent::NextRound => app.next_round(),
            AppEvent::ToggleTarget => app.toggle_guess_target(),
            AppEvent::Tick => {
                // Just update the display
            }
        }
    }
    Ok(())
}
