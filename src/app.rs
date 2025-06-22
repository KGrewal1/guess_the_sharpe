use crate::dist::{DAYS, Stats, gen_random_dist, plot_data};
use compact_str::CompactString;
use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;

#[derive(Debug, Clone, PartialEq)]
pub enum AppMode {
    Display,
    Guessing(Guess),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Guess {
    pub state: GuessState,
    pub target: GuessTarget,
    pub current_guess: CompactString,
    pub score: u32,
    pub last_guess: Option<f64>,
    pub guess_was_correct: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GuessState {
    WaitingForGuess,
    ShowingResult,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GuessTarget {
    Sample,
    Actual,
}

impl GuessTarget {
    pub fn name(&self) -> &'static str {
        match self {
            GuessTarget::Sample => "Sample",
            GuessTarget::Actual => "Actual",
        }
    }
}

pub struct App {
    pub running: bool,
    pub rng: ChaCha20Rng,
    pub plot_data: [(f64, f64); DAYS],
    pub stats: Stats,
    pub mode: AppMode,
}

impl App {
    pub fn new(mode: AppMode) -> Self {
        let mut rng = ChaCha20Rng::from_os_rng();
        let (sample, stats) = gen_random_dist(&mut rng);

        let plot_data = plot_data(&sample);

        Self {
            running: true,
            rng,
            plot_data,
            stats,
            mode,
        }
    }

    pub fn quit(&mut self) {
        self.running = false;
    }

    pub fn recalc(&mut self) {
        let (sample, stats) = gen_random_dist(&mut self.rng);
        self.plot_data = plot_data(&sample);
        self.stats = stats;

        if let AppMode::Guessing(ref mut guess) = self.mode {
            guess.state = GuessState::WaitingForGuess;
            guess.current_guess.clear();
            guess.last_guess = None;
            guess.guess_was_correct = false;
            // Note: we don't reset score here as it should persist across rounds
        };
    }

    pub fn add_char_to_guess(&mut self, c: char) {
        if let AppMode::Guessing(ref mut guess) = self.mode {
            if guess.state == GuessState::WaitingForGuess {
                // Only allow digits and decimal point in the guess
                if c.is_ascii_digit() || c == '.' {
                    guess.current_guess.push(c);
                }
            }
        }
    }

    pub fn remove_char_from_guess(&mut self) {
        if let AppMode::Guessing(ref mut guess) = self.mode {
            if guess.state == GuessState::WaitingForGuess {
                guess.current_guess.pop();
            }
        }
    }

    pub fn toggle_guess_target(&mut self) {
        if let AppMode::Guessing(ref mut guess) = self.mode {
            guess.target = match guess.target {
                GuessTarget::Sample => GuessTarget::Actual,
                GuessTarget::Actual => GuessTarget::Sample,
            };
        }
    }

    pub fn submit_guess(&mut self) {
        if let AppMode::Guessing(ref mut guess) = self.mode {
            if guess.state == GuessState::WaitingForGuess {
                if let Ok(parsed_guess) = guess.current_guess.parse::<f64>() {
                    guess.last_guess = Some(parsed_guess);

                    let sharpe_error = self.stats.sharpe_error;

                    // Choose the target value based on guess_target
                    let target_value = match guess.target {
                        GuessTarget::Sample => self.stats.sample_sharpe,
                        GuessTarget::Actual => self.stats.acc_sharpe,
                    };

                    // Check if guess is within error bounds of target
                    // sample sharpe error ~ 1 std dev - use 0.12 std dev to get about 10% of the dist
                    if (parsed_guess - target_value).abs() <= 0.12 * sharpe_error {
                        guess.score += 1;
                        guess.guess_was_correct = true;
                    } else {
                        guess.guess_was_correct = false;
                    }

                    guess.state = GuessState::ShowingResult;
                }
            }
        }
    }

    pub fn next_round(&mut self) {
        if let AppMode::Guessing(ref guess) = self.mode {
            if guess.state == GuessState::ShowingResult {
                self.recalc();
            }
        }
    }
}
