use crate::app::{App, AppMode, GuessState};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    symbols,
    text::{Line, Span},
    widgets::{Axis, Block, Borders, Chart, Dataset, GraphType, Paragraph},
};

pub fn ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3), // Stats section
            Constraint::Min(0),    // Chart section
            Constraint::Length(3), // Instructions section
        ])
        .split(f.area());

    // Stats section
    match app.mode {
        AppMode::Display => render_display_stats(f, app, chunks[0]),
        AppMode::Guessing => render_guessing_stats(f, app, chunks[0]),
    }

    // Chart section
    render_chart(f, app, chunks[1]);

    // Instructions section
    match app.mode {
        AppMode::Display => render_display_instructions(f, chunks[2]),
        AppMode::Guessing => render_guessing_instructions(f, app, chunks[2]),
    }
}

fn render_display_stats(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let sharpe_error = app.stats.sharpe_error;
    let mean_return = app.stats.sample_mean;
    let min_return = app.stats.sample_min;
    let max_return = app.stats.sample_max;

    let stats_text = vec![Line::from(vec![
        Span::styled("Actual Sharpe: ", Style::default().fg(Color::Yellow)),
        Span::styled(
            format!("{:.4}", app.stats.acc_sharpe),
            Style::default().fg(Color::Green),
        ),
        Span::raw("  "),
        Span::styled("Sample Sharpe: ", Style::default().fg(Color::Yellow)),
        Span::styled(
            format!("{:.4}", app.stats.sample_sharpe),
            Style::default().fg(Color::Cyan),
        ),
        Span::styled(
            format!(" ±{:.4}", sharpe_error),
            Style::default().fg(Color::Gray),
        ),
        Span::raw("  "),
        Span::styled("Mean: ", Style::default().fg(Color::Yellow)),
        Span::styled(
            format!("{:.6}", mean_return),
            Style::default().fg(Color::White),
        ),
        Span::raw("  "),
        Span::styled("Min: ", Style::default().fg(Color::Yellow)),
        Span::styled(
            format!("{:.4}", min_return),
            Style::default().fg(Color::Red),
        ),
        Span::raw("  "),
        Span::styled("Max: ", Style::default().fg(Color::Yellow)),
        Span::styled(
            format!("{:.4}", max_return),
            Style::default().fg(Color::Green),
        ),
    ])];

    let stats_paragraph = Paragraph::new(stats_text)
        .block(Block::default().borders(Borders::ALL).title("Statistics"))
        .style(Style::default().fg(Color::White));

    f.render_widget(stats_paragraph, area);
}

fn render_guessing_stats(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let stats_text = match app.guess_state {
        GuessState::WaitingForGuess => {
            vec![Line::from(vec![
                Span::styled("Your guess: ", Style::default().fg(Color::Yellow)),
                Span::styled(
                    &app.current_guess,
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::UNDERLINED),
                ),
                Span::raw("   "),
                Span::styled(
                    format!("Score: {}", app.score),
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("   "),
                Span::styled("Target: ", Style::default().fg(Color::Yellow)),
                Span::styled(
                    app.get_guess_target_name(),
                    Style::default()
                        .fg(Color::Magenta)
                        .add_modifier(Modifier::BOLD),
                ),
            ])]
        }
        GuessState::ShowingResult => {
            let result_color = if app.guess_was_correct {
                Color::Green
            } else {
                Color::Red
            };
            let result_text = if app.guess_was_correct {
                "CORRECT!"
            } else {
                "INCORRECT"
            };
            let sharpe_error = app.stats.sharpe_error;

            // Get the target value that was being guessed
            let target_value = match app.guess_target {
                crate::app::GuessTarget::Sample => app.stats.sample_sharpe,
                crate::app::GuessTarget::Actual => app.stats.acc_sharpe,
            };

            vec![Line::from(vec![
                Span::styled("Guess: ", Style::default().fg(Color::Yellow)),
                Span::styled(
                    format!("{:.4}", app.last_guess.unwrap_or(0.0)),
                    Style::default().fg(Color::White),
                ),
                Span::raw(" | "),
                Span::styled("Target: ", Style::default().fg(Color::Yellow)),
                Span::styled(
                    format!("{:.4}", target_value),
                    Style::default().fg(Color::Magenta),
                ),
                Span::styled(
                    format!(" ({}) ±{:.4}", app.get_guess_target_name(), sharpe_error),
                    Style::default().fg(Color::Gray),
                ),
                Span::raw(" | "),
                Span::styled(
                    result_text,
                    Style::default()
                        .fg(result_color)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" | "),
                Span::styled("Actual: ", Style::default().fg(Color::Yellow)),
                Span::styled(
                    format!("{:.4}", app.stats.acc_sharpe),
                    Style::default().fg(Color::LightCyan),
                ),
                Span::raw(" | "),
                Span::styled("Sample: ", Style::default().fg(Color::Yellow)),
                Span::styled(
                    format!("{:.4}", app.stats.sample_sharpe),
                    Style::default().fg(Color::LightCyan),
                ),
                Span::raw(" | "),
                Span::styled(
                    format!("Score: {}", app.score),
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
            ])]
        }
    };

    let stats_paragraph = Paragraph::new(stats_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Guessing Game"),
        )
        .style(Style::default().fg(Color::White));

    f.render_widget(stats_paragraph, area);
}

fn render_chart(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let plot_data = app.plot_data;

    // Find min and max values for scaling
    let min_y = plot_data
        .iter()
        .map(|(_, y)| *y)
        .fold(f64::INFINITY, f64::min);
    let max_y = plot_data
        .iter()
        .map(|(_, y)| *y)
        .fold(f64::NEG_INFINITY, f64::max);
    let max_x = plot_data.len() as f64;

    let datasets = vec![
        Dataset::default()
            .marker(symbols::Marker::Braille)
            .style(Style::default().fg(Color::Cyan))
            .graph_type(GraphType::Line)
            .data(&plot_data),
    ];

    let chart = Chart::new(datasets)
        .block(
            Block::default()
                .title("Cumulative Returns Plot")
                .borders(Borders::ALL),
        )
        .x_axis(
            Axis::default()
                .title("Day")
                .style(Style::default().fg(Color::Gray))
                .bounds([0.0, max_x])
                .labels(vec![
                    Span::styled("0", Style::default().add_modifier(Modifier::BOLD)),
                    Span::styled(
                        format!("{:.0}", max_x / 2.0),
                        Style::default().add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        format!("{:.0}", max_x),
                        Style::default().add_modifier(Modifier::BOLD),
                    ),
                ]),
        )
        .y_axis(
            Axis::default()
                .title("Cum Ret")
                .style(Style::default().fg(Color::Gray))
                .bounds([min_y, max_y])
                .labels(vec![
                    Span::styled(
                        format!("{:.3}", min_y),
                        Style::default().add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        format!("{:.3}", (min_y + max_y) / 2.0),
                        Style::default().add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        format!("{:.3}", max_y),
                        Style::default().add_modifier(Modifier::BOLD),
                    ),
                ]),
        );

    f.render_widget(chart, area);
}

fn render_display_instructions(f: &mut Frame, area: ratatui::layout::Rect) {
    let instructions = vec![Line::from(vec![
        Span::styled("Press ", Style::default().fg(Color::White)),
        Span::styled(
            "'r'",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" to recalculate, ", Style::default().fg(Color::White)),
        Span::styled(
            "'q'",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" to quit", Style::default().fg(Color::White)),
    ])];

    let instructions_paragraph = Paragraph::new(instructions)
        .block(Block::default().borders(Borders::ALL).title("Controls"))
        .style(Style::default().fg(Color::White));

    f.render_widget(instructions_paragraph, area);
}

fn render_guessing_instructions(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let instructions = match app.guess_state {
        GuessState::WaitingForGuess => {
            vec![Line::from(vec![
                Span::styled(
                    "Type your Sharpe ratio guess and press ",
                    Style::default().fg(Color::White),
                ),
                Span::styled(
                    "Enter",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(" to submit. Press ", Style::default().fg(Color::White)),
                Span::styled(
                    "'t'",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(" to toggle target, ", Style::default().fg(Color::White)),
                Span::styled(
                    "'q'",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(" to quit", Style::default().fg(Color::White)),
            ])]
        }
        GuessState::ShowingResult => {
            vec![Line::from(vec![
                Span::styled("Press ", Style::default().fg(Color::White)),
                Span::styled(
                    "'n'",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(" for next round, ", Style::default().fg(Color::White)),
                Span::styled(
                    "'q'",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(" to quit", Style::default().fg(Color::White)),
            ])]
        }
    };

    let instructions_paragraph = Paragraph::new(instructions)
        .block(Block::default().borders(Borders::ALL).title("Controls"))
        .style(Style::default().fg(Color::White));

    f.render_widget(instructions_paragraph, area);
}
