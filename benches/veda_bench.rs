use criterion::{black_box, criterion_group, criterion_main, Criterion};
use veda::{hmac::murmur3::murmur3_32, topk::heavykeeper::HeavyKeeper};

pub fn criterion_benchmark(c: &mut Criterion) {
    use std::fs::File;
    use std::io::prelude::*;

    c.bench_function("murmur3", |b| {
        let data = b"Lorem ipsum dolor sit amet, consectetur adipisicing elit";
        b.iter(|| {
            murmur3_32(black_box(data), 0);
        })
    });

    c.bench_function("bench add", |b| {
        let mut bfp = File::open("data/heavykeeper.bench.data").unwrap();
        let mut bdata = String::new();
        bfp.read_to_string(&mut bdata).unwrap();
        let data_lines: Vec<String> = bdata
            .split("\n")
            .into_iter()
            .map(|x| x.to_string())
            .collect();
        let mut topk = HeavyKeeper::new(10, 1000, 5, 0.9);
        let mut index = 0;

        b.iter(|| {
            topk.add(&data_lines[index % 1000], 1);
            index += 1;
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
