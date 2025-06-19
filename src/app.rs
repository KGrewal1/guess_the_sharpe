use crate::dist::{DAYS, gen_random_dist};
use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AppMode {
    Display,
    Guessing,
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

pub struct App {
    pub running: bool,
    pub rng: ChaCha20Rng,
    pub sample: [f64; DAYS],
    pub acc_sharpe: f64,
    pub sample_sharpe: f64,
    pub mode: AppMode,
    pub guess_state: GuessState,
    pub guess_target: GuessTarget,
    pub current_guess: String,
    pub score: u32,
    pub last_guess: Option<f64>,
    pub guess_was_correct: bool,
}

impl App {
    pub fn new(mode: AppMode) -> Self {
        let mut rng = ChaCha20Rng::from_os_rng();
        let (sample, acc_sharpe, sample_sharpe) = gen_random_dist(&mut rng);

        Self {
            running: true,
            rng,
            sample,
            acc_sharpe,
            sample_sharpe,
            mode,
            guess_state: GuessState::WaitingForGuess,
            guess_target: GuessTarget::Sample,
            current_guess: String::new(),
            score: 0,
            last_guess: None,
            guess_was_correct: false,
        }
    }

    pub fn quit(&mut self) {
        self.running = false;
    }

    pub fn recalc(&mut self) {
        let (sample, acc_sharpe, sample_sharpe) = gen_random_dist(&mut self.rng);
        self.sample = sample;
        self.acc_sharpe = acc_sharpe;
        self.sample_sharpe = sample_sharpe;

        if self.mode == AppMode::Guessing {
            self.guess_state = GuessState::WaitingForGuess;
            self.current_guess.clear();
            self.last_guess = None;
        }
    }

    pub fn add_char_to_guess(&mut self, c: char) {
        if self.mode == AppMode::Guessing && self.guess_state == GuessState::WaitingForGuess && (c.is_ascii_digit() || c == '.' || c == '-') {
            self.current_guess.push(c);
        }
    }

    pub fn remove_char_from_guess(&mut self) {
        if self.mode == AppMode::Guessing && self.guess_state == GuessState::WaitingForGuess {
            self.current_guess.pop();
        }
    }
    pub fn toggle_guess_target(&mut self) {
        if self.mode == AppMode::Guessing && self.guess_state == GuessState::WaitingForGuess {
            self.guess_target = match self.guess_target {
                GuessTarget::Sample => GuessTarget::Actual,
                GuessTarget::Actual => GuessTarget::Sample,
            };
        }
    }

    pub fn submit_guess(&mut self) {
        if self.mode == AppMode::Guessing && self.guess_state == GuessState::WaitingForGuess {
            if let Ok(guess) = self.current_guess.parse::<f64>() {
                self.last_guess = Some(guess);

                // Calculate sample sharpe error: sqrt((1 + sharpe^2 / 2) / T)
                let sharpe_error = self.get_sharpe_error();

                // Choose the target value based on guess_target
                let target_value = match self.guess_target {
                    GuessTarget::Sample => self.sample_sharpe,
                    GuessTarget::Actual => self.acc_sharpe,
                };

                // Check if guess is within error bounds of target
                // sample sharpe error ~ 1 std dev - use 0.12 std dev to get about 10% of the dist
                if (guess - target_value).abs() <= 0.12 * sharpe_error {
                    self.score += 1;
                    self.guess_was_correct = true;
                } else {
                    self.guess_was_correct = false;
                }

                self.guess_state = GuessState::ShowingResult;
            }
        }
    }

    pub fn next_round(&mut self) {
        if self.mode == AppMode::Guessing && self.guess_state == GuessState::ShowingResult {
            self.recalc();
        }
    }

    pub fn get_sharpe_error(&self) -> f64 {
        ((1.0 + self.sample_sharpe.powi(2) / 2.0) / DAYS as f64).sqrt() * (252.0_f64.sqrt())
    }

    pub fn get_guess_target_name(&self) -> &'static str {
        match self.guess_target {
            GuessTarget::Sample => "Sample",
            GuessTarget::Actual => "Actual",
        }
    }

    pub fn get_plot_data(&self) -> Vec<(f64, f64)> {
        let mut cumulative_return = 0.0;
        self.sample
            .iter()
            .enumerate()
            .map(|(i, &value)| {
                cumulative_return += value;
                (i as f64, cumulative_return)
            })
            .collect()
    }
}
