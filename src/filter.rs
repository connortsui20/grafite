use crate::PairwiseIndependentHasher;
use std::ops::RangeBounds;
use vers_vecs::EliasFanoVec;

/// The Grafite Range Filter.
#[derive(Debug, Clone)]
pub struct RangeFilter {
    /// The hash function used to encode the hash values.
    pub hasher: PairwiseIndependentHasher,
    /// A succinct encoding of a non-decreasing sequence of integer hash values.
    pub ef: EliasFanoVec,
}

/// The `RangeFilter` must be built on items that are able to be turned into a 64-bit integer.
impl RangeFilter {
    /// Creates a new `RangeFilter` given a slice of values.
    pub fn new<I>(values: I, hasher: PairwiseIndependentHasher) -> Self
    where
        I: Iterator<Item = u64>,
    {
        // Hash all items in the input set.
        let mut hashes: Vec<u64> = values.map(|x| hasher.local_hash(x)).collect();

        // Sort and then remove all duplicates.
        hashes.sort_unstable();
        hashes.dedup();

        assert!(hashes[hashes.len() - 1] < hasher.reduced_universe_size());

        Self {
            hasher,
            ef: EliasFanoVec::from_slice(&hashes),
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

        match self.ef.predecessor(end_hash) {
            // If the end hash has no predecessor, then there can't be any elements in the set less
            // than the input range end, which means there is no element in between start and end.
            None => false,
            // If the predecessor is less than the start hash, then there cannot be any elements in
            // between start and end.
            Some(predecessor) => predecessor >= start_hash,
        }
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
        (num_elements as u64 * max_interval) as f64 / self.hasher.reduced_universe_size() as f64
    }

    /// Returns the amount of space required to store this `RangeFilter` on the heap.
    ///
    /// Internally, this function simply calls [`heap_size`](EliasFanoVec::heap_size) on the inner
    /// [`EliasFanoVec`] structure.
    pub fn heap_size(&self) -> usize {
        self.ef.heap_size()
    }
}
