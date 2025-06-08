use grafite::{PairwiseIndependentHasher, RangeFilter};

#[test]
fn test_basic() {
    let values = [1, 2, 3, 7, 8, 9, 15, 20];

    let hasher = PairwiseIndependentHasher::new(values.len(), 0.01, 20).unwrap();

    let rf = RangeFilter::new(values.iter().copied(), hasher);

    assert!(rf.query(0..20));
    assert!(rf.query(0..10));
    assert!(rf.query(0..5));
    assert!(rf.query(3..5));
    assert!(!rf.query(4..5));
    assert!(!rf.query(4..6));
    assert!(!rf.query(4..7));
    assert!(rf.query(4..8));
    assert!(rf.query(4..10));
    assert!(!rf.query(10..14));
    assert!(!rf.query(10..15));
    assert!(rf.query(10..16));
}
