//! Divan-harness comparison for PhastFT vs. RustFFT; forward direction only; `f32`/`f64`,
//! sizes `2^6..=2^22`.
//!
//! The structure of these benches are not meant to benchmark the implementations
//! in terms of runtime, but rather in terms of memory usage and allocations.
//!
//! Note this builds the planner (and RustFFT scratch) inside the
//! timed portion, so divan's `AllocProfiler` can report each library's
//! memory footprint as part of its output.
//!
//! To run, use the following:
//! ```sh
//! cargo bench --bench divan_compare
//! ```

use divan::counter::ItemsCount;
use divan::{AllocProfiler, Bencher};
use num_traits::{Float, Zero};
use phastft::options::Options;
use phastft::planner::{Direction, PlannerDit32, PlannerDit64};
use phastft::{fft_32_dit_with_planner_and_opts, fft_64_dit_with_planner_and_opts};
use rand::distr::StandardUniform;
use rand::prelude::Distribution;
use utilities::rustfft::num_complex::Complex;
use utilities::rustfft::{FftNum, FftPlanner};

mod common;
use common::{interleaved_complex, split_complex};

#[global_allocator]
static ALLOC: AllocProfiler = AllocProfiler::system();

fn main() {
    divan::main();
}

const LENGTHS: &[usize] = &[
    1 << 6,
    1 << 7,
    1 << 8,
    1 << 9,
    1 << 10,
    1 << 11,
    1 << 12,
    1 << 13,
    1 << 14,
    1 << 15,
    1 << 16,
    1 << 17,
    1 << 18,
    1 << 19,
    1 << 20,
    1 << 21,
    1 << 22,
];

struct Phastft;
struct RustFft;

trait SplitFft<T> {
    fn forward(reals: &mut [T], imags: &mut [T]);
}

trait InterleavedFft<T> {
    fn forward(signal: &mut [Complex<T>]);
}

impl SplitFft<f32> for Phastft {
    fn forward(reals: &mut [f32], imags: &mut [f32]) {
        let n = reals.len();
        let planner = PlannerDit32::new(n);
        let opts = Options::guess_options(n);
        fft_32_dit_with_planner_and_opts(reals, imags, Direction::Forward, &planner, &opts);
    }
}

impl SplitFft<f64> for Phastft {
    fn forward(reals: &mut [f64], imags: &mut [f64]) {
        let n = reals.len();
        let planner = PlannerDit64::new(n);
        let opts = Options::guess_options(n);
        fft_64_dit_with_planner_and_opts(reals, imags, Direction::Forward, &planner, &opts);
    }
}

impl<T: FftNum> InterleavedFft<T> for RustFft {
    fn forward(signal: &mut [Complex<T>]) {
        let mut planner = FftPlanner::<T>::new();
        let fft = planner.plan_fft_forward(signal.len());
        let mut scratch = vec![Complex::<T>::zero(); fft.get_inplace_scratch_len()];
        fft.process_with_scratch(signal, &mut scratch);
    }
}

#[divan::bench(types = [f32, f64], args = LENGTHS)]
fn phastft_forward<T>(bencher: Bencher, n: usize)
where
    T: Float,
    Phastft: SplitFft<T>,
    StandardUniform: Distribution<T>,
{
    bencher
        .counter(ItemsCount::new(n))
        .with_inputs(|| split_complex::<T>(n))
        .bench_local_refs(|(reals, imags)| Phastft::forward(reals, imags));
}

#[divan::bench(types = [f32, f64], args = LENGTHS)]
fn rustfft_forward<T>(bencher: Bencher, n: usize)
where
    T: Float,
    RustFft: InterleavedFft<T>,
    StandardUniform: Distribution<T>,
{
    bencher
        .counter(ItemsCount::new(n))
        .with_inputs(|| interleaved_complex::<T>(n))
        .bench_local_refs(|signal| RustFft::forward(signal));
}
