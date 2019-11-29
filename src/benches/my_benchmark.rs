use criterion::{black_box, criterion_group, criterion_main, Criterion};
use kvs::{KvStore, KvsEngine, SledKvsEngine};
use rand::distributions::Alphanumeric;
use rand::prelude::*;
use std::iter::Zip;

use tempfile::TempDir;

fn generate_random_strings(rng: &mut ThreadRng, size: usize, item_size_max: usize) -> Vec<String> {
    let mut result: Vec<String> = vec![];

    for _ in 0..size {
        let item_size: usize = (rng.next_u32() % item_size_max as u32 + 1) as usize;
        let val: String = rng.sample_iter(&Alphanumeric).take(item_size).collect();
        result.push(val)
    }

    result
}

pub fn kvs_write_benchmark(c: &mut Criterion) {
    let temp_dir = TempDir::new().unwrap();
    let mut engine: KvStore = KvStore::open(temp_dir.path()).unwrap();
    let mut rng = rand::thread_rng();
    let random_keys: Vec<String> = generate_random_strings(&mut rng, 100, 100000);
    let random_values: Vec<String> = generate_random_strings(&mut rng, 100, 100000);

    c.bench_function("kvs write", move |b| {
        b.iter(|| {
            let indx: usize = (rng.next_u32() % 100) as usize;
            engine
                .set(random_keys[indx].clone(), random_values[indx].clone())
                .unwrap();
        })
    });
}

pub fn kvs_read_benchmark(c: &mut Criterion) {
    let temp_dir = TempDir::new().unwrap();
    let mut engine: KvStore = KvStore::open(temp_dir.path()).unwrap();
    let mut rng = rand::thread_rng();
    let random_keys: Vec<String> = generate_random_strings(&mut rng, 100, 100000);
    let random_values: Vec<String> = generate_random_strings(&mut rng, 100, 100000);
    let pair = random_keys.iter().zip(random_values.iter());

    for (key, value) in pair {
        engine.set(key.clone(), value.clone()).unwrap()
    }

    c.bench_function("kvs read", move |b| {
        b.iter(|| {
            let indx: usize = (rng.next_u32() % 100) as usize;
            engine.get(random_keys[indx].clone()).unwrap();
        })
    });
}

pub fn sled_write_benchmark(c: &mut Criterion) {
    let temp_dir = TempDir::new().unwrap();
    let mut engine: SledKvsEngine = SledKvsEngine::open(temp_dir.path()).unwrap();
    let mut rng = rand::thread_rng();
    let random_keys: Vec<String> = generate_random_strings(&mut rng, 100, 100000);
    let random_values: Vec<String> = generate_random_strings(&mut rng, 100, 100000);

    c.bench_function("kvs write", move |b| {
        b.iter(|| {
            let indx: usize = (rng.next_u32() % 100) as usize;
            engine
                .set(random_keys[indx].clone(), random_values[indx].clone())
                .unwrap();
        })
    });
}

pub fn sled_read_benchmark(c: &mut Criterion) {
    let temp_dir = TempDir::new().unwrap();
    let mut engine: SledKvsEngine = SledKvsEngine::open(temp_dir.path()).unwrap();
    let mut rng = rand::thread_rng();
    let random_keys: Vec<String> = generate_random_strings(&mut rng, 100, 100000);
    let random_values: Vec<String> = generate_random_strings(&mut rng, 100, 100000);
    let pair = random_keys.iter().zip(random_values.iter());

    for (key, value) in pair {
        engine.set(key.clone(), value.clone()).unwrap()
    }

    c.bench_function("kvs read", move |b| {
        b.iter(|| {
            let indx: usize = (rng.next_u32() % 100) as usize;
            engine.get(random_keys[indx].clone()).unwrap();
        })
    });
}

criterion_group!(
    benches,
    kvs_write_benchmark,
    kvs_read_benchmark,
    sled_write_benchmark,
    sled_read_benchmark
);
criterion_main!(benches);
