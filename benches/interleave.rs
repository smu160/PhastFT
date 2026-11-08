//! Requires the `bench-internals` feature to access private functions in the
//! crate root. Run with:
//!
//! ```sh
//! cargo bench --bench interleave --features bench-internals
//! ```

use criterion::{criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion};
use phastft::complex_nums::{combine_re_im, deinterleave_complex};

mod common;
use common::{bench_at_sizes, groups, ids, interleaved_complex, split_complex, throughput_complex};

// Keep the extended size sweep used by the in-place experiments.
const LENGTHS: &[usize] = &[
    6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
];

#[cfg(feature = "parallel")]
use phastft::complex_nums::{
    deinterleave_inplace_parallel as deinterleave_inplace,
    interleave_inplace_parallel as interleave_inplace,
};
#[cfg(not(feature = "parallel"))]
use phastft::complex_nums::{
    deinterleave_inplace_sequential as deinterleave_inplace,
    interleave_inplace_sequential as interleave_inplace,
};

macro_rules! deinterleave_bench {
    ($name:ident, $float:ty, $deinter_fn:ident, $group:expr) => {
        fn $name(c: &mut Criterion) {
            bench_at_sizes(
                c,
                $group,
                LENGTHS,
                throughput_complex::<$float>,
                |g, len| {
                    g.bench_function(BenchmarkId::new(ids::DEINTERLEAVE, len), |b| {
                        b.iter_batched(
                            || interleaved_complex::<$float>(len),
                            |input| $deinter_fn(&input),
                            BatchSize::SmallInput,
                        );
                    });
                },
            );
        }
    };
}

macro_rules! combine_bench {
    ($name:ident, $float:ty, $group:expr) => {
        fn $name(c: &mut Criterion) {
            bench_at_sizes(
                c,
                $group,
                LENGTHS,
                throughput_complex::<$float>,
                |g, len| {
                    g.bench_function(BenchmarkId::new(ids::COMBINE_RE_IM, len), |b| {
                        b.iter_batched(
                            || split_complex::<$float>(len),
                            |(reals, imags)| combine_re_im::<$float>(&reals, &imags),
                            BatchSize::SmallInput,
                        );
                    });
                },
            );
        }
    };
}

deinterleave_bench!(
    deinterleave_f32,
    f32,
    deinterleave_complex,
    groups::KERNEL_DEINTERLEAVE_F32
);
deinterleave_bench!(
    deinterleave_f64,
    f64,
    deinterleave_complex,
    groups::KERNEL_DEINTERLEAVE_F64
);
combine_bench!(combine_f32, f32, groups::KERNEL_COMBINE_RE_IM_F32);
combine_bench!(combine_f64, f64, groups::KERNEL_COMBINE_RE_IM_F64);

const BLOCK_SIZE_BYTES: usize = 8 * 1024;

macro_rules! inplace_bench {
    ($name:ident, $float:ty, $transform:ident, $group:expr) => {
        fn $name(c: &mut Criterion) {
            let block_size = BLOCK_SIZE_BYTES / size_of::<$float>();
            bench_at_sizes(
                c,
                $group,
                LENGTHS,
                throughput_complex::<$float>,
                |g, len| {
                    if len < block_size * 2 {
                        return;
                    }
                    g.bench_function(BenchmarkId::new(stringify!($transform), len), |b| {
                        b.iter_batched(
                            || interleaved_complex::<$float>(len),
                            |mut data| {
                                $transform(
                                    bytemuck::cast_slice_mut::<_, $float>(&mut data),
                                    block_size,
                                )
                            },
                            BatchSize::SmallInput,
                        );
                    });
                },
            );
        }
    };
}

inplace_bench!(
    benchmark_interleave_inplace_f32,
    f32,
    interleave_inplace,
    "Interleave inplace f32"
);
inplace_bench!(
    benchmark_interleave_inplace_f64,
    f64,
    interleave_inplace,
    "Interleave inplace f64"
);
inplace_bench!(
    benchmark_deinterleave_inplace_f32,
    f32,
    deinterleave_inplace,
    "Deinterleave inplace f32"
);
inplace_bench!(
    benchmark_deinterleave_inplace_f64,
    f64,
    deinterleave_inplace,
    "Deinterleave inplace f64"
);

criterion_group!(
    benches,
    deinterleave_f32,
    deinterleave_f64,
    combine_f32,
    combine_f64,
    benchmark_interleave_inplace_f32,
    benchmark_interleave_inplace_f64,
    benchmark_deinterleave_inplace_f32,
    benchmark_deinterleave_inplace_f64,
);
criterion_main!(benches);
