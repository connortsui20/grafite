//! Utility and helper functions for hashing and prime number generation.

use rand::prelude::*;
use std::ops::Range;

/// The number of iterations to run the Miller-Rabin primality test.
const ITERATIONS: usize = 128;

/// Generates a random 64-bit number that is within the input `range`.
///
/// # Panics
///
/// Panics if the range is empty.
pub fn gen_random(range: Range<u64>) -> u64 {
    rand::rng().random_range(range)
}

/// Deterministically checks if a number is prime.
#[allow(dead_code)]
pub fn is_prime(n: u64) -> bool {
    match n {
        0 | 1 => false,
        2 => true,
        _ if n % 2 == 0 => false,
        _ => !(1..)
            .map(|x| 2 * x + 1)
            .take_while(|&x| x * x <= n)
            .any(|factor| n % factor == 0),
    }
}

/// Generates a random 64-bit (potentially) prime number that is within the input range.
///
/// This function will generate a random number and then use the Miller-Rabin primality test to
/// check if the number generated is prime. If it returns `true`, then it will return that number as
/// the candidate prime number. Otherwise, it will generate a new random number and try again.
///
/// # Panics
///
/// Panics if the range is empty.
pub fn gen_prime(range: Range<u64>) -> u64 {
    let mut rng = rand::rng();

    loop {
        let attempt = rng.random_range(range.clone());

        if miller_rabin::is_prime(&attempt, ITERATIONS) {
            return attempt;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_prime() {
        let primes = [2, 3, 5, 7, 11, 13, 17, 19];

        assert!(primes.iter().copied().all(is_prime));
    }
}
