//! End-to-end layout comparison. Build with --release --features bench-internals.
//! Arguments: precision min_log2 max_log2 threads serial|default forward|inverse samples ms [mode]
//! Planning, fresh input clones and reusable scratch allocation are not timed.
//! Each sample accumulates at least `ms` of FFT time in batches bounded by 1 MiB.
//! In-place blocks are clamped to N for small transforms.
//! BENCH_MEMORY=1 runs one selected mode on a single input, without validation
//! copies or the timing harness, for /usr/bin/time peak-RSS measurements.
//! BENCH_REUSE_INPUT=1 reuses input allocations, resetting their contents outside
//! timing, instead of allocating fresh input batches (the repository default).
//! Select `api` to benchmark the public interleaved wrapper. Modes can be comma
//! separated. BENCH_TWIDDLE_MODE=auto|precomputed|low-memory selects the plan.

use num_complex::Complex;
use phastft::{
    complex_nums::*,
    options::Options,
    planner::{Direction, PlannerDit32, PlannerDit64, TwiddleMode},
};
use std::{
    hint::black_box,
    time::{Duration, Instant},
};

const MODES: &[&str] = &[
    "oop_old",
    "oop_direct",
    "oop_reuse",
    "ip_serial",
    "ip_parallel",
    "api",
];

struct Config {
    min: u32,
    max: u32,
    threads: usize,
    serial: bool,
    direction: Direction,
    samples: usize,
    duration: Duration,
    mode: Option<String>,
    twiddles: TwiddleMode,
}

macro_rules! benchmark {
    ($name:ident, $float:ty, $plan:ty, $fft:path, $interleaved:path) => {
        fn $name(c: &Config) {
            for log_n in c.min..=c.max {
                let n = 1usize << log_n;
                let plan = <$plan>::with_twiddle_mode(n, c.twiddles);
                let mut opts = Options::guess_options(n);
                if c.serial {
                    opts.multithreaded_bit_reversal = false;
                    opts.smallest_parallel_chunk_size = usize::MAX;
                }
                let mut input: Vec<Complex<$float>> = (0..n)
                    .map(|i| {
                        // Deterministic, bounded input, avoiding a trivial impulse or constant signal.
                        let x = (i as u64)
                            .wrapping_mul(6364136223846793005)
                            .wrapping_add(1442695040888963407);
                        Complex::new(
                            ((x >> 32) as u32 as f64 / u32::MAX as f64 - 0.5) as $float,
                            (x as u32 as f64 / u32::MAX as f64 - 0.5) as $float,
                        )
                    })
                    .collect();
                let block = (8192 / size_of::<$float>()).min(n);
                let batch = ((1 << 20) / (2 * n * size_of::<$float>())).clamp(1, 256);
                let modes: Vec<&str> = MODES
                    .iter()
                    .copied()
                    .filter(|m| match c.mode.as_deref() {
                        None => *m != "api",
                        Some(s) => s.split(',').any(|selected| selected == *m),
                    })
                    .collect();
                assert!(!modes.is_empty(), "Unknown mode");
                let mut scratch_re = if modes.contains(&"oop_reuse") {
                    vec![0.0; n]
                } else {
                    Vec::new()
                };
                let mut scratch_im = scratch_re.clone();
                let mut run = |mode: &str, data: &mut [Complex<$float>]| {
                    match mode {
                        "api" => $interleaved(data, c.direction, &plan, &opts),
                        "oop_old" | "oop_direct" => {
                            let (mut re, mut im) = deinterleave_complex(data);
                            $fft(&mut re, &mut im, c.direction, &plan, &opts);
                            if mode == "oop_old" {
                                data.copy_from_slice(&combine_re_im(&re, &im));
                            } else {
                                combine_re_im_into(&re, &im, bytemuck::cast_slice_mut(data));
                            }
                        }
                        "oop_reuse" => {
                            deinterleave_into(
                                bytemuck::cast_slice(data),
                                &mut scratch_re,
                                &mut scratch_im,
                            );
                            $fft(&mut scratch_re, &mut scratch_im, c.direction, &plan, &opts);
                            combine_re_im_into(
                                &scratch_re,
                                &scratch_im,
                                bytemuck::cast_slice_mut(data),
                            );
                        }
                        "ip_serial" | "ip_parallel" => {
                            let flat: &mut [$float] = bytemuck::cast_slice_mut(data);
                            if mode == "ip_serial" {
                                deinterleave_inplace_sequential(flat, block);
                            } else {
                                deinterleave_inplace_parallel(flat, block);
                            }
                            let (re, im) = flat.split_at_mut(n);
                            $fft(re, im, c.direction, &plan, &opts);
                            if mode == "ip_serial" {
                                interleave_inplace_sequential(flat, block);
                            } else {
                                interleave_inplace_parallel(flat, block);
                            }
                        }
                        _ => unreachable!(),
                    }
                    black_box(data);
                };
                if std::env::var_os("BENCH_MEMORY").is_some() {
                    assert_eq!(modes.len(), 1, "Memory measurement requires a single mode");
                    assert_eq!(c.min, c.max, "Memory measurement requires a single size");
                    run(modes[0], &mut input);
                    continue;
                }
                // Check both conversion directions against the split-layout FFT, outside timing.
                let (mut re, mut im) = deinterleave_complex(&input);
                $fft(&mut re, &mut im, c.direction, &plan, &opts);
                let expected = combine_re_im(&re, &im);
                drop(re);
                drop(im);
                for mode in &modes {
                    let mut actual = input.clone();
                    run(mode, &mut actual);
                    assert_eq!(actual, expected, "{mode}, N={n}");
                }
                drop(expected);
                let reuse_input = std::env::var_os("BENCH_REUSE_INPUT").is_some();
                let mut reused_data: Vec<_> = if reuse_input {
                    (0..batch).map(|_| input.clone()).collect()
                } else {
                    Vec::new()
                };
                // Deterministic shuffle per sample avoids systematic order/temperature bias.
                let mut random = 0x123456789abcdef_u64;
                for sample in 0..c.samples {
                    let mut order = modes.clone();
                    for j in (1..order.len()).rev() {
                        random ^= random << 13;
                        random ^= random >> 7;
                        random ^= random << 17;
                        order.swap(j, random as usize % (j + 1));
                    }
                    for mode in order {
                        let mut elapsed = Duration::ZERO;
                        let mut transforms = 0;
                        while elapsed < c.duration || transforms == 0 {
                            let mut fresh_data;
                            let data = if reuse_input {
                                for d in &mut reused_data {
                                    d.copy_from_slice(&input);
                                }
                                &mut reused_data
                            } else {
                                fresh_data = (0..batch).map(|_| input.clone()).collect::<Vec<_>>();
                                &mut fresh_data
                            };
                            let start = Instant::now();
                            for d in data {
                                run(mode, d);
                            }
                            elapsed += start.elapsed();
                            transforms += batch;
                            // Input destruction stays outside timing; internal allocations do not.
                        }
                        println!(
                            "{},{log_n},{n},{},{},{:?},{:?},{mode},{sample},{transforms},{:.3}",
                            stringify!($float),
                            c.threads,
                            if c.serial { "serial" } else { "default" },
                            c.direction,
                            plan.twiddle_mode(),
                            elapsed.as_nanos() as f64 / transforms as f64
                        );
                    }
                }
                eprintln!("finished {} 2^{log_n}", stringify!($float));
            }
        }
    };
}

benchmark!(
    bench32,
    f32,
    PlannerDit32,
    phastft::fft_f32_dit_with_planner_and_opts,
    phastft::fft_f32_dit_interleaved_with_planner_and_opts
);
benchmark!(
    bench64,
    f64,
    PlannerDit64,
    phastft::fft_f64_dit_with_planner_and_opts,
    phastft::fft_f64_dit_interleaved_with_planner_and_opts
);

fn main() {
    let args: Vec<String> = std::env::args().collect();
    assert!(
        args.len() >= 9,
        "precision min_log2 max_log2 threads serial|default forward|inverse samples ms [mode]"
    );
    assert!(matches!(args[5].as_str(), "serial" | "default"));
    let c = Config {
        min: args[2].parse().unwrap(),
        max: args[3].parse().unwrap(),
        threads: args[4].parse().unwrap(),
        serial: args[5] == "serial",
        direction: match args[6].as_str() {
            "forward" => Direction::Forward,
            "inverse" => Direction::Inverse,
            _ => panic!("direction"),
        },
        samples: args[7].parse().unwrap(),
        duration: Duration::from_millis(args[8].parse().unwrap()),
        mode: args.get(9).cloned(),
        twiddles: match std::env::var("BENCH_TWIDDLE_MODE")
            .as_deref()
            .unwrap_or("auto")
        {
            "auto" => TwiddleMode::Auto,
            "precomputed" => TwiddleMode::Precomputed,
            "low-memory" => TwiddleMode::LowMemory,
            other => panic!("Invalid BENCH_TWIDDLE_MODE: {other}"),
        },
    };
    rayon::ThreadPoolBuilder::new()
        .num_threads(c.threads)
        .build_global()
        .unwrap();
    println!("precision,log2,n,threads,fft,direction,twiddles,mode,sample,transforms,ns");
    match args[1].as_str() {
        "32" => bench32(&c),
        "64" => bench64(&c),
        _ => panic!("precision"),
    }
}
