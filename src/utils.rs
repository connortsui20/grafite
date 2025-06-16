//! Utility and helper functions for hashing and prime number generation.

use rand::prelude::*;
use std::ops::Range;

/// The number of iterations to run the Miller-Rabin primality test.
const ITERATIONS: usize = 128;

/// Generates a random 64-bit (potentially) prime number that is within the input range.
///
/// This function will generate a random number and then use the Miller-Rabin primality test to
/// check if the number generated is prime. If it returns `true`, then it will return that number as
/// the candidate prime number. Otherwise, it will generate a new random number and try again.
///
/// # Panics
///
/// Panics if the range is empty.
pub fn gen_prime(rng: &mut ThreadRng, range: Range<u64>) -> u64 {
    loop {
        let attempt = rng.random_range(range.clone());

        if miller_rabin::is_prime(&attempt, ITERATIONS) {
            return attempt;
        }
    }
}
