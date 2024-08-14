use criterion::{black_box, criterion_group, criterion_main, Criterion};
use delta_encoding::{IndexedData, SimpleDirectDeltaEncoding};

fn criterion_benchmark(c: &mut Criterion) {
    let current_dir = std::env::current_dir().expect("Failed to get current directory");
    let data_path = current_dir.join("benches").join("bench_files");
    let mut og_path = data_path.clone();
    og_path.push("text_1.txt");
    let mut diff_path = data_path.clone();
    diff_path.push("text_2.txt");

    let original_data_from_file = &[IndexedData::new(0, std::fs::read(og_path).expect("Failed to read original_data.txt"))];
    let diff_data_from_file = &[IndexedData::new(0, std::fs::read(diff_path.clone()).expect("Failed to read new_data.txt"))];

    let mut sdd = SimpleDirectDeltaEncoding::new(diff_data_from_file);
    let diff_data = sdd.patch(original_data_from_file);
    
    c.bench_function("create patch", |b| b.iter(|| {
        let mut sdd = SimpleDirectDeltaEncoding::new(diff_data_from_file);
        let _ = sdd.patch(original_data_from_file);
    }));
    c.bench_function("apply patch", |b| b.iter(|| {
        let mut sdd2 = SimpleDirectDeltaEncoding::new(diff_data_from_file);
        let _ = sdd2.apply_patch(black_box(&diff_data)).ok();
    }));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);