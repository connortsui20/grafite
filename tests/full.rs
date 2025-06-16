use grafite::{PairwiseIndependentHasher, RangeFilter};
use rand::Rng;
use rayon::prelude::*;
use std::sync::atomic::{AtomicUsize, Ordering};

const NUM_ITERATIONS: usize = 1_000_000_000;

fn bench(num_elements: usize, bits_per_key: u8, max_interval: u64) {
    println!(
        "\n\n\
        Beginning benchmark of {num_elements} elements, {bits_per_key} bits per key,\
        and a maximum query interval of {max_interval}\
        \n\n"
    );

    let mut values: Vec<u64> = (0..num_elements)
        .into_par_iter()
        .map(|_| rand::rng().random())
        .collect();

    println!("Finished generating {num_elements} values");

    let hasher =
        PairwiseIndependentHasher::new_with_space_budget(num_elements, bits_per_key, max_interval)
            .unwrap();

    let rf = RangeFilter::new(values.iter().copied(), hasher);

    println!(
        "Finished constructing RangeFilter, taking {} bytes of space",
        rf.heap_size()
    );
    println!(
        "Expected false positive rate: {}",
        rf.false_positive_rate(num_elements, max_interval)
    );

    values.sort_unstable();

    println!("Finished sorting {num_elements} values");

    let false_positives = AtomicUsize::new(0);

    println!("Running {NUM_ITERATIONS} queries");

    (0..NUM_ITERATIONS).into_par_iter().for_each(|_| {
        let x: u64 = rand::rng().random();

        let res = rf.query(x..x + max_interval);

        if !res {
            debug_assert!(!values.contains(&x));
        } else if values.binary_search(&x).is_err() {
            // If the range filter says that the value is present, but it is actually not...
            false_positives.fetch_add(1, Ordering::AcqRel);
        }
    });

    let false_positives = false_positives.load(Ordering::Acquire);
    let measured_epsilon = false_positives as f64 / NUM_ITERATIONS as f64;

    println!("Measured false positive rate: {measured_epsilon}");
}

#[test]
fn full_benches() {
    bench(200_000_000, 12, 1 << 5);
    bench(200_000_000, 16, 1 << 10);
}
