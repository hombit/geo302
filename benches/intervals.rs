#![cfg(feature = "ripe-geo-embedded")]
use criterion::{black_box, criterion_group, Criterion};
use geo302::geo::ripe_geo::RipeGeoImpl;
use std::net::Ipv4Addr;

criterion_group!(benches_interval_tree, bench_interval_trees);

pub fn bench_interval_trees(c: &mut Criterion) {
    let keys: [u32; 5] = [
        [0u8, 0, 0, 0],
        [127, 0, 0, 1],
        [80, 94, 184, 70],
        [93, 180, 26, 112],
        [128, 174, 199, 60],
    ]
    .into_iter()
    .map(|v| {
        let ip: Ipv4Addr = v.into();
        ip.into()
    })
    .collect::<Vec<u32>>()
    .try_into()
    .unwrap();

    {
        let btree_map = RipeGeoImpl::from_embedded().into_interval_btree_maps().0;
        c.bench_function("IntervalBTreeMap::get", |b| {
            b.iter(|| {
                for key in keys.into_iter() {
                    btree_map.get(black_box(key));
                }
            })
        });
    }

    {
        let vecs = RipeGeoImpl::from_embedded().into_interval_vecs().0;
        c.bench_function("IntervalVec::get", |b| {
            b.iter(|| {
                for key in keys.into_iter() {
                    vecs.get(black_box(key));
                }
            })
        });
    }

    {
        let intervals_rw = RipeGeoImpl::from_embedded().into_intervals().0.into_rw();
        c.bench_function("Intervals::Rw::get", |b| {
            b.iter(|| {
                for key in keys.into_iter() {
                    intervals_rw.get(black_box(key));
                }
            })
        });
    }

    {
        let intervals_ro = RipeGeoImpl::from_embedded().into_intervals().0.into_ro();
        c.bench_function("Intervals::Ro::get", |b| {
            b.iter(|| {
                for key in keys.into_iter() {
                    intervals_ro.get(black_box(key));
                }
            })
        });
    }
}
