use criterion::{Criterion, criterion_group, criterion_main};

fn bench_random_generate(c: &mut Criterion) {
    c.bench_function("generate_random_128", |b| {
        b.iter(|| {
            // call binary via library
            let _ = genix_lib::generate::generate_many("random", 128, 1, None, false, None);
        })
    });
}

fn bench_passphrase_generate(c: &mut Criterion) {
    c.bench_function("generate_passphrase_4", |b| {
        b.iter(|| {
            let _ = genix_lib::generate::generate_many("passphrase", 4, 1, None, false, None);
        })
    });
}

criterion_group!(benches, bench_random_generate, bench_passphrase_generate);
criterion_main!(benches);
