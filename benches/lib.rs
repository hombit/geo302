#[cfg(feature = "ripe-geo-embedded")]
use criterion::criterion_main;

mod intervals;

#[cfg(feature = "ripe-geo-embedded")]
criterion_main!(intervals::benches_interval_tree);

#[cfg(not(feature = "ripe-geo-embedded"))]
fn main() {}
