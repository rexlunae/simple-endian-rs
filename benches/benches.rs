//use criterion::{criterion_group, criterion_main, Criterion};
use bencher::{Bencher, benchmark_group, benchmark_main};

benchmark_group!(benches, bench_integer_be, bench_integer_le, bench_integer_ne, bench_fp_be, bench_fp_le
    , bench_fp_ne, base_endian_test_be, base_endian_test_le, base_endian_test_ne, base_endian_test_structured);
//benchmark_group!(benches, bench_integer_be);
benchmark_main!(benches);

//criterion_group!(benches, bench_integer_be);
//criterion_main!(benches);

use simple_endian::{BigEndian, LittleEndian};

fn bench_integer_be(b: &mut Bencher) {
    b.iter(|| {
        let mut a = BigEndian::from(1234567890);
        for _ in 0..10 {
            a += BigEndian::from(101010);
            a &= BigEndian::from(0xf0f0f0);
            a *= BigEndian::from(123);
            a /= BigEndian::from(543);
        }
        println!("{}", a);
    });
}
fn bench_integer_le(b: &mut Bencher) {
    b.iter(|| {
        let mut a = LittleEndian::from(1234567890);
        for _ in 0..10 {
            a += LittleEndian::from(101010);
            a &= LittleEndian::from(0xf0f0f0);
            a *= LittleEndian::from(123);
            a /= LittleEndian::from(543);
        }
        println!("{}", a);
    });
}
fn bench_integer_ne(b: &mut Bencher) {
    b.iter(|| {
        let mut a = 1234567890;
        for _ in 0..10 {
            a += 101010;
            a &= 0xf0f0f0;
            a *= 123;
            a /= 543;
        }
        println!("{}", a);
    });
}


fn bench_fp_be(b: &mut Bencher) {
    b.iter(|| {
        let mut a = BigEndian::from(1234567890.1);
        for _ in 0..10 {
            a += BigEndian::from(101010.0);
            a *= BigEndian::from(123.0);
            a /= BigEndian::from(543.0);
        }
        println!("{}", a);
    });
}

fn bench_fp_le(b: &mut Bencher) {
    b.iter(|| {
        let mut a = LittleEndian::from(1234567890.1);
        for _ in 0..10 {
            a += LittleEndian::from(101010.0);
            a *= LittleEndian::from(123.0);
            a /= LittleEndian::from(543.0);
        }
        println!("{}", a);
    });
}

fn bench_fp_ne(b: &mut Bencher) {
    b.iter(|| {
        let mut a = 1234567890.1;
        for _ in 0..10 {
            a += 101010.0;
            a *= 123.0;
            a /= 543.0;
        }
        println!("{}", a);
    });
}


fn base_endian_test_be(b: &mut Bencher) {
    b.iter(|| {
        for _ in 0..1000 {
            let a = i32::from_be(0xa5a5a5);
            println!("{}", a);
        }
    });
}

fn base_endian_test_le(b: &mut Bencher) {
    b.iter(|| {
        for _ in 0..1000 {
            let a = i32::from_le(0xa5a5a5);
            println!("{}", a);
        }
    });
}

fn base_endian_test_ne(b: &mut Bencher) {
    b.iter(|| {
        for _ in 0..1000 {
            let a = 0xa5a5a5_i32;
            println!("{}", a);
        }
    });
}

fn base_endian_test_structured(b: &mut Bencher) {
    b.iter(|| {
        for _ in 0..1000 {
            let a = LittleEndian::from(0xa5a5a5_i32);
            println!("{}", a);
        }
    });
}
