//! PhastFT Bluestein vs rustfft Bluestein at a matched power-of-2 inner size.
//!
//! rustfft's `BluesteinsAlgorithm` accepts any inner `M >= 2N-1`; to compare
//! implementation-to-implementation we give it the SAME `M = next_pow2(2N-1)`
//! PhastFT is constrained to (read from `planner.inner_fft_len()` /
//! recomputed identically here), holding the convolution size constant.
//!
//! Both sides pre-allocate scratch outside the timing loop and reuse it
//! (`iter_batched` regenerates only the input signal per sample), so no
//! allocation is measured. Compile-verify with `cargo bench --bench bluestein
//! --no-run`; measurement runs are user-initiated.

use criterion::{criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion};
use num_traits::Zero;
use phastft::options::Options;
use phastft::planner::{Direction, PlannerBluestein32, PlannerBluestein64};
use phastft::{fft_f32_bluestein_with_planner_and_opts, fft_f64_bluestein_with_planner_and_opts};
use utilities::rustfft::algorithm::BluesteinsAlgorithm;
use utilities::rustfft::num_complex::Complex;
use utilities::rustfft::{Fft, FftPlanner};

mod common;
use common::{
    groups, ids, interleaved_complex, make_group, split_complex, throughput_complex,
    BLUESTEIN_LENGTHS,
};

macro_rules! bluestein_compare {
    ($name:ident, $T:ty, $planner:ty, $phastft_fn:ident, $dir:expr, $rustfft_plan:ident, $group:expr) => {
        fn $name(c: &mut Criterion) {
            let mut group = make_group(c, $group);
            for &len in BLUESTEIN_LENGTHS {
                group.throughput(throughput_complex::<$T>(len));

                // --- PhastFT Bluestein (planner + scratch reused) ---
                let planner = <$planner>::new(len);
                let m = planner.inner_fft_len();
                let opts = Options::guess_options(m);
                let mut sr = vec![0.0; m];
                let mut si = vec![0.0; m];
                group.bench_function(BenchmarkId::new(ids::PHASTFT_BLUESTEIN, len), |b| {
                    b.iter_batched(
                        || split_complex::<$T>(len),
                        |(mut re, mut im)| {
                            $phastft_fn(&mut re, &mut im, $dir, &planner, &opts, &mut sr, &mut si);
                            std::hint::black_box((&mut re, &mut im));
                        },
                        BatchSize::SmallInput,
                    );
                });

                // --- rustfft Bluestein, matched M (algorithm + scratch reused) ---
                let inner = FftPlanner::<$T>::new().$rustfft_plan(m);
                let fft = BluesteinsAlgorithm::new(len, inner);
                let mut scratch = vec![Complex::<$T>::zero(); fft.get_inplace_scratch_len()];
                group.bench_function(BenchmarkId::new(ids::RUSTFFT_BLUESTEIN, len), |b| {
                    b.iter_batched(
                        || interleaved_complex::<$T>(len),
                        |mut signal| {
                            fft.process_with_scratch(&mut signal, &mut scratch);
                            std::hint::black_box(&mut signal);
                        },
                        BatchSize::SmallInput,
                    );
                });
            }
            group.finish();
        }
    };
}

bluestein_compare!(
    fwd_f64,
    f64,
    PlannerBluestein64,
    fft_f64_bluestein_with_planner_and_opts,
    Direction::Forward,
    plan_fft_forward,
    groups::C2C_BLUESTEIN_FORWARD_F64
);
bluestein_compare!(
    inv_f64,
    f64,
    PlannerBluestein64,
    fft_f64_bluestein_with_planner_and_opts,
    Direction::Inverse,
    plan_fft_inverse,
    groups::C2C_BLUESTEIN_INVERSE_F64
);
bluestein_compare!(
    fwd_f32,
    f32,
    PlannerBluestein32,
    fft_f32_bluestein_with_planner_and_opts,
    Direction::Forward,
    plan_fft_forward,
    groups::C2C_BLUESTEIN_FORWARD_F32
);
bluestein_compare!(
    inv_f32,
    f32,
    PlannerBluestein32,
    fft_f32_bluestein_with_planner_and_opts,
    Direction::Inverse,
    plan_fft_inverse,
    groups::C2C_BLUESTEIN_INVERSE_F32
);

criterion_group!(benches, fwd_f64, inv_f64, fwd_f32, inv_f32);
criterion_main!(benches);
