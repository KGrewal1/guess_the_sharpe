use rand::distr::StandardUniform;
use rand::prelude::*;
use rand_chacha::ChaCha20Rng;
use rand_distr::num_traits::Float;
use rand_distr::{Distribution, Normal};

pub const DAYS: usize = 504; // Number of trading days in 2 years - 252 days per year

/// Generates a random Sharpe ratio in the range of -3 to 3.
fn gen_rand_sharpe(rng: &mut ChaCha20Rng) -> f64 {
    let sharpe: f64 = (rng.sample::<f64, _>(StandardUniform) * 6.0) - 3.0; // Generate a number between -3 and 3
    sharpe
}

fn gen_return_series(sharpe: f64, rng: &mut ChaCha20Rng) -> [f64; DAYS] {
    let mut returns = [0.; DAYS];
    // annual sharpe = mu / sigma - assume sigma = 1.0 so annual mu = sharpe
    // in daily terms this means mu = sharpe / 252 and sigma = 1.0 / sqrt(252)
    let normal = Normal::new(sharpe / 252., 252.0.sqrt().recip()).unwrap();

    returns.iter_mut().for_each(|x| *x = normal.sample(rng));
    returns
}

fn calc_sample_sharpe(sample: [f64; DAYS]) -> f64 {
    let sample_mu = sample.iter().sum::<f64>() / DAYS as f64;
    let sample_var = sample.iter().map(|x| (x - sample_mu).powi(2)).sum::<f64>() / DAYS as f64;
    let sample_std = sample_var.sqrt();
    // Annualize the Sharpe ratio: multiply mean by 252 and std by sqrt(252)

    (sample_mu * 252.0) / (sample_std * 252.0_f64.sqrt())
}

pub fn gen_random_dist(rng: &mut ChaCha20Rng) -> ([f64; DAYS], f64, f64) {
    let acc_sharpe = gen_rand_sharpe(rng);
    let returns = gen_return_series(acc_sharpe, rng);
    let sample_sharpe = calc_sample_sharpe(returns);
    (returns, acc_sharpe, sample_sharpe)
}
