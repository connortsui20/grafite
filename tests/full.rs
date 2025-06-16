use grafite::{PairwiseIndependentHasher, RangeFilter};
use rand::Rng;
use rayon::prelude::*;
use std::sync::atomic::{AtomicUsize, Ordering};

/// The number of range queries to make.
///
/// This number is high in order to achieve a statistically significant false positive rate.
const NUM_ITERATIONS: usize = 1_000_000;

fn bench(num_elements: usize, bits_per_key: u8, max_interval: u64) {
    eprintln!(
        "\n\n\
        Beginning benchmark of {num_elements} elements, {bits_per_key} bits per key,\
        and a maximum query interval of {max_interval}\
        \n\n"
    );

    eprintln!("Generating {num_elements} random values");

    let mut values: Vec<u64> = (0..num_elements)
        .into_par_iter()
        .map(|_| rand::rng().random())
        .collect();

    let hasher =
        PairwiseIndependentHasher::new_with_space_budget(num_elements, bits_per_key, max_interval)
            .unwrap();

    eprintln!("Constructing the range filter over {num_elements} values");

    let rf = RangeFilter::new(values.iter().copied(), hasher);

    eprintln!(
        "The range filter takes up {} bytes of space.\nExpected false positive rate of {}",
        rf.ef.heap_size(),
        rf.hasher.false_positive_rate(num_elements, max_interval)
    );

    // Sort the values to make verifaction faster with binary search.
    values.sort_unstable();

    eprintln!("Running {NUM_ITERATIONS} queries");
    let false_positives = AtomicUsize::new(0);

    (0..NUM_ITERATIONS).into_par_iter().for_each(|_| {
        let x: u64 = rand::rng().random();

        let res = rf.query(x..x + max_interval);

        if !res {
            debug_assert!(!values.contains(&x));
        } else if values.binary_search(&x).is_err() {
            // If the range filter says that the value is present, but it is actually not...
            false_positives.fetch_add(1, Ordering::Relaxed);
        }
    });

    let measured_epsilon = false_positives.load(Ordering::Relaxed) as f64 / NUM_ITERATIONS as f64;

    eprintln!("Measured false positive rate: {measured_epsilon}");
    assert!(
        measured_epsilon <= 1.05 * rf.hasher.false_positive_rate(num_elements, max_interval),
        "False positive rate was more than 5% higher than expected"
    )
}

#[test]
fn full_benches() {
    bench(200_000, 12, 1 << 5);
    bench(200_000, 16, 1 << 10);
    bench(200_000_000, 12, 1 << 5);
    bench(200_000_000, 16, 1 << 10);
}
