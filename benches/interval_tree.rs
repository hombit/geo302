use criterion::{black_box, criterion_group, Criterion};
use geo302::geo::ripe_geo::RipeGeoImpl;
use geo302::interval_tree::IntervalTreeMap;
use itertools::Itertools;
use std::collections::BTreeMap;
use std::net::Ipv4Addr;

criterion_group!(benches_interval_tree, bench_interval_trees);

#[derive(Clone)]
struct BtreeMapImpl<K, V, S = K>(IntervalTreeMap<K, V, S>);

impl<K, V, S> BtreeMapImpl<K, V, S>
where
    K: std::ops::Add<S, Output = K> + Ord,
    K: Copy + std::fmt::Debug,
    S: Copy,
{
    fn get(&self, key: K) -> Option<&V> {
        self.0.get(key)
    }
}

#[derive(Clone)]
struct VecImpl<K, V, S = K> {
    keys: Vec<K>,
    sizes_values: Vec<(S, V)>,
}

impl<K, V, S> VecImpl<K, V, S>
where
    K: std::ops::Add<S, Output = K> + Ord,
    K: Copy + std::fmt::Debug,
    S: Copy,
{
    fn from_btree_map_impl(value: BtreeMapImpl<K, V, S>) -> Self {
        let btree_map: BTreeMap<_, _> = value.0.into();
        let (keys, sizes_values) = btree_map.into_iter().unzip();
        Self { keys, sizes_values }
    }

    fn get(&self, key: K) -> Option<&V> {
        match self.keys.binary_search(&key) {
            Ok(index) => Some(&self.sizes_values[index].1),
            Err(0) => None,
            Err(mut index) => {
                index -= 1;
                let (size, value) = &self.sizes_values[index];
                if key < self.keys[index] + *size {
                    Some(value)
                } else {
                    None
                }
            }
        }
    }
}

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

    let ripe_geo_impl = RipeGeoImpl::from_embedded();
    let btree_map_impl = BtreeMapImpl(ripe_geo_impl.into_interval_tree_maps().0);
    c.bench_function("BTreeMapImpl::get", |b| {
        b.iter(|| {
            for key in keys.into_iter() {
                btree_map_impl.get(black_box(key));
            }
        })
    });

    let vec_impl = VecImpl::from_btree_map_impl(btree_map_impl);
    c.bench_function("VecImpl::get", |b| {
        b.iter(|| {
            for key in keys.into_iter() {
                vec_impl.get(black_box(key));
            }
        })
    });
}
