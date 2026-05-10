use mdv::ast::{Block, BlockId, Inline};
use mdv::virt::{visible_range, HeightCache};

fn make_blocks(n: usize) -> Vec<(BlockId, Block)> {
    (0..n)
        .map(|i| {
            (
                BlockId(i as u64),
                Block::Paragraph(vec![Inline::Text(format!("block {}", i))]),
            )
        })
        .collect()
}

#[test]
fn pad_extends_range_in_both_directions() {
    let blocks = make_blocks(100);
    let cache = HeightCache::default();
    let (s, e) = visible_range(&blocks, &cache, 1000.0, 400.0, 5);
    assert!(s > 0, "should not start at 0 with offset 1000");
    assert!(e <= blocks.len());
    let (s2, e2) = visible_range(&blocks, &cache, 1000.0, 400.0, 0);
    assert!(s <= s2);
    assert!(e >= e2);
}

#[test]
fn measured_height_overrides_estimate() {
    let blocks = make_blocks(10);
    let mut cache = HeightCache::default();
    cache.set_measured(BlockId(0), 9999.0);
    let (s, _) = visible_range(&blocks, &cache, 5000.0, 400.0, 0);
    // First block alone is now 9999px tall, so offset 5000 still falls in block 0.
    assert_eq!(s, 0);
}
