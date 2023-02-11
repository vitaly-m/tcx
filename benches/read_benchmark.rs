use criterion::{criterion_group, criterion_main, Criterion};


fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("tcx", |b| b.iter(|| {
        let file = std::fs::File::open("test_resources/test.tcx.xml").unwrap();
        let mut reader = std::io::BufReader::new(file);
        let _result = quick_tcx::read(&mut reader).unwrap();
    }));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);