use criterion::criterion_main;

#[cfg(feature = "ripe-geo-embedded")]
mod interval_tree;

criterion_main!(interval_tree::benches_interval_tree);
