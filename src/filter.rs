use crate::PairwiseIndependentHasher;
use std::ops::RangeBounds;
use vers_vecs::EliasFanoVec;

/// The Grafite Range Filter.
#[derive(Debug, Clone)]
pub struct RangeFilter {
    /// The hash function used to encode the hash values.
    hasher: PairwiseIndependentHasher,

    /// A succinct encoding of a non-decreasing sequence of integer hash values.
    ef: EliasFanoVec,
}

/// The `RangeFilter` must be built on items that are able to be turned into a 64-bit integer.
impl RangeFilter {
    /// Creates a new `RangeFilter` given a slice of values.
    ///
    /// # Panics
    ///
    /// Panics if the input `values` iterator is empty, or if the input hasher is invalid (hashes a
    /// value to a number greater than the reduced universe size).
    pub fn new<I>(values: I, hasher: PairwiseIndependentHasher) -> Self
    where
        I: Iterator<Item = u64>,
    {
        // Hash all items in the input set.
        let mut hashed_values: Vec<u64> = values.map(|x| hasher.local_hash(x)).collect();

        assert!(
            !hashed_values.is_empty(),
            "Cannot create a range filter with no elements"
        );

        // Sort and then remove all duplicates.
        hashed_values.sort_unstable();
        hashed_values.dedup();

        assert!(
            hashed_values.last().expect("We checked emptyness above")
                < &hasher.reduced_universe_size(),
            "A hashed value was greater than the size of the reduced universe {}",
            hasher.reduced_universe_size()
        );

        Self {
            hasher,
            ef: EliasFanoVec::from_slice(&hashed_values),
        }
    }

    /// Checks if there are any elements within the given range among the original input set.
    pub fn query<R>(&self, range: R) -> bool
    where
        R: RangeBounds<u64>,
    {
        let start = match range.start_bound() {
            std::ops::Bound::Included(&i) => i,
            std::ops::Bound::Excluded(_) => unreachable!("Somehow had an exclusive start bound"),
            std::ops::Bound::Unbounded => 0,
        };

        let end = match range.end_bound() {
            std::ops::Bound::Included(&i) => i,
            std::ops::Bound::Excluded(&e) => e - 1,
            std::ops::Bound::Unbounded => u64::MAX,
        };

        let start_hash = self.hasher.local_hash(start);
        let end_hash = self.hasher.local_hash(end);

        // If the start hash is greater than the end hash, then the range has wrapped around due to
        // the reduced universe. Thus we can just check the min and max hashes to see if there is an
        // element between the endpoints.
        if start_hash > end_hash {
            return self.min_hash() <= end_hash || self.max_hash() >= start_hash;
        }

        self.ef
            .predecessor(end_hash)
            .is_some_and(|predecessor| predecessor >= start_hash)
    }

    /// Gets the minimum hash value in the sorted hash codes.
    fn min_hash(&self) -> u64 {
        self.ef.get_unchecked(0)
    }

    /// Gets the maximum hash value in the sorted hash codes.
    fn max_hash(&self) -> u64 {
        self.ef.get_unchecked(self.ef.len() - 1)
    }
    /// Returns the false positive rate, epsilon.
    ///
    /// The false positive rate is determined by the hash function used, the maximum range of values
    /// queried, and the total number of distinct values inside the range filter.
    pub fn false_positive_rate(&self, num_elements: usize, max_interval: u64) -> f64 {
        // The false positive rate is equal to nL / r.
        let nl = (num_elements as u64 * max_interval) as f64;
        let r = self.hasher.reduced_universe_size() as f64;

        nl / r
    }

    /// Returns the amount of space required to store this `RangeFilter` on the heap.
    ///
    /// Internally, this function simply calls [`heap_size`](EliasFanoVec::heap_size) on the inner
    /// [`EliasFanoVec`] structure.
    pub fn heap_size(&self) -> usize {
        self.ef.heap_size()
    }
}
