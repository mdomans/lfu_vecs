#[macro_use]
extern crate criterion;
extern crate bytes;
extern crate lfu_vecs;
extern crate rand;

use bytes::Bytes;
use criterion::{Criterion, Fun};
use rand::{Rng, SeedableRng, XorShiftRng};

fn insert_and_lookup_standard(mut n: u64) {
    let mut rng: XorShiftRng = SeedableRng::from_seed([1981, 1986, 2003, 2011]);
    let mut hash_map = ::std::collections::HashMap::new();

    while n != 0 {
        let key: String = (0..10).map(|_| rand::random::<u8>() as char).collect();
        if rng.gen::<bool>() {
            let value = Bytes::from((0..10).map(|_| rand::random::<u8>()).collect::<Vec<u8>>());
            hash_map.insert(key, value);
        } else {
            hash_map.get(&key);
        }
        n -= 1;
    }
}

fn insert_and_lookup_lfu(mut n: u64) {
    let mut rng: XorShiftRng = SeedableRng::from_seed([1981, 1986, 2003, 2011]);
    let mut hash_map = lfu_vecs::LFU::new().max_size(100000);

    while n != 0 {
        let key: String = (0..10).map(|_| rand::random::<u8>() as char).collect();
        if rng.gen::<bool>() {
            let value = Bytes::from((0..10).map(|_| rand::random::<u8>()).collect::<Vec<u8>>());
            hash_map.insert(key, value);
        } else {
            hash_map.get(&key);
        }
        n -= 1;
    }
}

fn insert_and_lookup_lfu_low_max_size(mut n: u64) {
    let mut rng: XorShiftRng = SeedableRng::from_seed([1981, 1986, 2003, 2011]);
    let mut hash_map = lfu_vecs::LFU::new().max_size(100);

    while n != 0 {
        let key: String = (0..10).map(|_| rand::random::<u8>() as char).collect();
        if rng.gen::<bool>() {
            let value = Bytes::from((0..10).map(|_| rand::random::<u8>()).collect::<Vec<u8>>());
            hash_map.insert(key, value);
        } else {
            hash_map.get(&key);
        }
        n -= 1;
    }
}

macro_rules! insert_lookup {
    ($fn:ident, $s:expr) => {
        fn $fn(c: &mut Criterion) {
            let lfu = Fun::new("lfu", |b, i| b.iter(|| insert_and_lookup_lfu(*i)));
            let lfu_constrained = Fun::new("lfu with low size", |b, i| b.iter(|| insert_and_lookup_lfu_low_max_size(*i)));
            let standard = Fun::new("standard", |b, i| b.iter(|| insert_and_lookup_standard(*i)));

            let functions = vec![lfu, lfu_constrained, standard];
            c.bench_functions(&format!("HashMap/{}", $s), functions, $s);
        }
    };
}

//insert_lookup!(insert_lookup_1000, 1000);
//insert_lookup!(insert_lookup_10000, 10000);
insert_lookup!(insert_lookup_100000, 100000);

criterion_group!(
    name = benches;
    config = Criterion::default();
    targets = insert_lookup_100000
);

criterion_main!(benches);
