# Changelog

All notable changes to this project will be documented in this file.

## [0.4.0-rc.1] - 2026-05-03

### Features

- Add the rustfmt.toml config actual
- Remove nightly requirement for portable SIMD
- Add Rayon to the f32 codepath
- Add a tunable for the smallest chunk size beyond which the input will not be split further
- Add initial experimental BRAVO impl
- Add DiT kernel wrappers to act as an inlining barrier, otherwise the entirety of DiT kernels ends up rolled up into one function on ARM and collapses under register pressure
- Add precision configuration to our own benchmark example
- Add precision configuration to benchmark scripts for rustfft and fftw
- Add a benchmark for planner
- Add interleaving/deinterleaving benchmark
- Add `criterion` benchmarks for bit reversal comparison
- Add real valued FFT/IFFT ([#105](https://github.com/QuState/PhastFT/pull/105))

### Bug Fixes

- Bump utilities version # and bump deps
- Add FMA to chunk 4 DIT kernel & fix organization
- Typo
- Remove dead code, fix py bindings, add missing docs
- Remove nightly from profiling script
- Fix build now that 'wide' renamed a bunch of its methods to align with portable_simd
- Fix YAML syntax
- Update GH Actions workflow to use stable rust
- Fix module documentation not being a proper module doc comment
- Ammend criterion benchmarks and bump criterion to v0.8.2
- Throughput should consider reals and imags
- Remove old C FFTW benches, improve methodology, and plots ([#117](https://github.com/QuState/PhastFT/pull/117))

### Other

- Placate clippy about floats and unused code
- Adopt the same rustfmt config as rust
- Require gfni target feature for the AVX-512 codepath

Boosts some of the benchmarks by 50% on Zen 4.

This instruction first appeared in 2019: https://en.wikipedia.org/wiki/Sunny_Cove_(microarchitecture)
Intel docs: https://builders.intel.com/docs/networkbuilders/galois-field-new-instructions-gfni-technology-guide-1-1639042826.pdf

On CPUs from before 2019 it's usually not worth it to use AVX-512 anyway because of the severe downlocking it induces.
- Don't multiversion on ARM

NEON is part of Aarch64 baseline instruction set, so multiversioning does nothing
- Proof-of-concept Rayon integration
- Experimental cross-half parallelization; regressed benchmarks
- Apply the same transformation to f32
- Dramatically lower COBRA multi-threading threshold now that thread spawning overhead is no longer a concern; benchmarks show improvement even at 15 on my machine but I'm being conservative for now, we'll need to auto-tune COBRA in the future because hardware varies so much
- Lower criterion sample size to 20 from its default of 100. Collecting 100 samples takes very long time because of our large number of benchmarks, and makes rayon confidently report 2-3% movements which are definitely just noise. This makes benchmarks both way faster and much more useful, we're not constantly drowning in noise anymore.
- Report throughput in bytes in addition to Melem/s (mlems per second)
- Try using chili as the parallelization backend instead of rayon. Benchmarks show that it regresses small sizes far less, but not enough to break even with the single-threaded implementation; but also regresses large sizes a lot. So it loses out to both rayon and single-threaded depending on the size and doesn't seem to be worth it.
- Do not customize release profile to match the typical library use
- Inline the generic DIT compute kernels manually now that parallelizing calls to them didn't work out
- Port to portable_simd proper instead of stable polyfill
- Adapt conversion to slice for fearless_simd
- Fully convert fft_dit_chunk_8_simd_f64 to fearless_simd
- Convert the rest of DIT functions to fearless_simd
- Wire up new DIT kernel function signatures to DIT process
- Dispatch to multiversioned fft_dit_chunk_2 via fearless_simd rather than multiversion
- Move SIMD dispatch one level higher so that it's definitely, positively not messing anything up
- Simpler SIMD loads
- Simpler SIMD stores
- Commit Cargo.lock for reproducible builds and reproducible benchmark results
- Consolidate BRAVO testing
- Change BRAVO impl from a generic to a macro in preparation for porting to fearless_simd
- Port BRAVO to fearless_simd
- Rename vecs to chunks
- Failed experiment: use dynamically sized storage instead of stack scratch space; collapses performance
- Don't explicitly pass the default value to BRAVO macro, use Default::default() instead
- Parametrize BRAVO impl on LANEs to allow adapting to native vector width
- Write down the rationale for using a macro
- Repoint fearless_simd dependency to git main
- Turn `x << 1` into `x * 2` and add comments on all the other uses of <<
- Detect the SIMD support level early and store it in the planner
- Wire up using the stored SIMD level to the FFT functions
- Upgrade fearless_simd dependency to get https://github.com/linebender/fearless_simd/pull/188
- Repeat the computation a configurable number of times
- Make iterations configurable, remove DiF codepath
- Targeted conversion to a for loop for just enough inlining
- Update a comment that's not really true anymore
- Hoist intermediate buffers out of the loop to avoid re-zeroing them every iteration
- Rustfft example benchmark: use process_with_scratch and keep scratch space initialization cost outside the measured time
- Port the same structure of configurable iteration count from our own harness to rustfft harness
- Properly report results
- Do the math internally and print average to simplify the benchmarking script
- Rename gen_random_signal to gen_random_signal_f64
- Apply the same changes to fftwrb example
- Double the chunk size for f64. Improves performance for very small and very large sizes, regresses medium sizes.
- README updates
- Re-add mention of non-power-of-2 algorithms
- Rewrite Features section
- Default to 16k parallelism threshold, don't require calling guess() for it
- Implement fft_{32,64}_interleaved_with_planner_and_opts, replacing the previous high-level-only API
- Round out the interleaved API
- Expose interleaving/deinterleaving functions as public but only when passing --cfg bench so they could be benchmarked
- Move SIMD level detection out of the function being benched
- Do not force inlining of BRAVO and CO-BRAVO implementations into algorithm selection function
- Another attempt at massaging f64 assembly; didn't work

### Refactor

- Restructure code into modules
- Move `cobra` to algorithms
- Switch from 'wide' from git to a crates.io version now that our changes have shipped
- Use a fast, non-secure PRNG in benchmarking harness
- Factor out the computational kernel for f64 DiT and use as_chunks instead of chunks_exact_mut for a perf boost
- Use as_chunks_mut everywhere else in the same file just on general principle
- Use parallel kernel only for the largest sizes when crossing halves
- Use rayon for parallelizing COBRA, helps performance of mid-sized FFTs by eliminating the thread spawning and termination overhead
- Make rayon dependency optional
- Rename parallel_join to something less technical
- Run tests both with and without all the features
- Move DIF-only function to DIF file
- Use mul_add with a float inversion instead of mul_neg_add

The code is more readable that way and also more portable to other SIMD libraries; wide is the only one with mul_neg_add. After my https://github.com/Lokathor/wide/pull/242 this generates identical assembly to mul_neg_add
- Make BRAVO impl generic over T as opposed to f64-only
- Use a branch with less forced inlining
- Rename functions for consistency
- Make BRAVO adapt to native SIMD width
- Refactor Dit planner to be generated by a macro for both 32 and 64 versions, same as Dif one
- Make f64 DiT planner dispatch on the SIMD level only once
- Make f32 DiT planner dispatch on the SIMD level only once
- Switch to released fearless_simd 0.4.0
- Use RustFFT more efficiently in benchmarks for a fair comparison
- Simplify BRAVO implementation by removing a redundant intermediate buffer
- Use a faster RNG for generating random signal
- Make planner fields pub(crate), they shouldn't be tampered with
- Move 'How is it so fast?' down in the README

to place usage examples more prominently since we also use the same README for docs.rs
- Rename utils to complex_nums
- Make all complex_nums expose fearless_simd API
- Move cfg() out of complex_nums.rs to avoid duplicating it all over the place
- Make BRAVO operating on chunks more explicit. Cuts down on sketchy/confusing-looking math and reduces bounds checks.
- Pass TILE_SIDE more explicitly to helper functions instead of reconstructing it from first principles
- Suppress counter-productive clippy lint
- Encode buffer length in the type system
- Simplify tile sizes. The change empirically improves performance on Zen4, which had an exception for small L1, but evidently it is not actually needed.
- Suppress useless clippy lint
- Factor COBRAVO out into its own function
- Use std::hint::black_box for more accurate benchmarks
- Use std::hint::black_box for more accurate small-size benchmarks
- Make the f64 codelet operate entirely in registers
- Make f32 codelet operate entirely in registers
- Use codelets unconditionally
- Mark mode as unused to suppress compiler warning (for now)
- Replace zip_low()+zip_high() with interleave(), should have better performance on avx2
- Polyfill interleave() until the upstream fearless_simd PR is merged
- Consolidate R2C/C2R API to one variant per direction

### Documentation

- Update README and fix formatting
- Update README and manifest
- Update doc comments for rayon
- Update doc comment
- Add a comment to the small size benchmarks pointing to examples/benchmark.rs
- Update README.md
- Update README.md
- Document running the newly added benchmark
- Clarify COBRAVO comment
- Add a comment on register splilling
- *(benches)* Remove old benchmark machine configuration
- Fix typo in README
- Update README, benchmark plots, and fix wording

### Performance

- Use `tzcnt` instruction in bit rev
- Add unrolled specialized kernels for bit rev
- Refactor DIT kernels to use 6 FMAs
- Use `mul_neg_add` in lieu of separate negation
- Add initial cache-blocked dit fft impl
- Eliminate intermediate storage in BRAVO. Doesn't affect performance - maybe LLVM eliminated it already?
- Optimize deinterleaving by avoiding a memset
- Add a FFT 32 codelet to fuse first 5 stages ([#101](https://github.com/QuState/PhastFT/pull/101))
- Add tiling to bravo to turn it into co-bravo
- Eliminate a lot of math by chunking the data in the type system
- Speed up random signal generation by removing per-element sqrt and sin_cos

    Generate real and imaginary parts directly from the RNG, then normalize
    so sum(re² + im²) == 1. This eliminates the per-element sqrt() and
    sin_cos() calls and the intermediate Vec allocation, replacing them with
    two cheap vectorizable linear passes over the data.
- Properly vectorize f32 codelet
- Reduce live set in f64 codelet to reduce register pressure
- Address the register spills in f32 codelet passes 0 and 1
- Tighten up stage 2 assembly in f32 codelet
- Reduce f64 codelet from 5 stages to 4, to reduce register pressure in the final stage
- Remove extra pass used for inverse transforms

### Testing

- Add test for DIT ifft and other test fixes
- Test with complex-nums feature but without parallelism so that we don't have to stop testing README snippets that Rust treats as doctest

### Miscellaneous Tasks

- Remove unused scalar kernels for DIT
- Drop lto=true from Cargo.toml

Not forcing LTO is more representative of how the crate would actually be used in third-party code.

It has no meaningful effect on benchmarks for PhastFT, but RustFFT is 15% faster on some sizes without LTO. This gives us a more accurate baseline. It also eases development - with LTO incremental build times are very long.
- Bump `criterion` to 0.8.0
- Remove all #[inline] directives to let the compiler make its own inlining decisions
- Clean up imports
- Update for the swapped order of arguments in simd_from
- Remove declared use of nightly in preparation to porting to fearless_simd
- Remove DIF and COBRA implementations
- Drop Bluestein's from plans, add R2C
- Update info on threading and complex numbers, mark Python as coming soon
- Update comparison vs RustFFT
- Remove wide dependency and SIMD twiddle generation
- Remove deleted Planner32/64 from bench
- Drop dynamic dispatch from interleaving/deinterleaving. The zip/unzip operations are part of base SSE; they appear to be already enough to saturate cache bandwidth. The compiler seems to unroll the loop and allow for ILP, so no wonder. AVX2 also requires a more expensive cross-lane shuffle so there's not a whole lot of benefit to it even in theory, and we get the exact same performance on benchmarks.
- Remove unused dit radix-2^2 kernels
- Bump `rand` and `bytemuck`
- Delete old, superseded codelets
- Remove dead (twiddles) code
- Remove old script for generating twiddles
- Drop unused import
- Remove codelet control from planner now that it is always beneficial
- Add `git-cliff` config and generated CHANGELOG.md
- Remove old/irrelevant benchmark plots from readme

### Revert

- Revert "Experimental cross-half parallelization; regressed benchmarks"

This reverts commit 5109c8d5331aa5b5188968f288933fa5afd467a5.
- Revert "Use parallel kernel only for the largest sizes when crossing halves"

This reverts commit 42cb42be546c521e2fd1b173b92740c8638b3bc4.
- Revert "Try using chili as the parallelization backend instead of rayon. Benchmarks show that it regresses small sizes far less, but not enough to break even with the single-threaded implementation; but also regresses large sizes a lot. So it loses out to both rayon and single-threaded depending on the size and doesn't seem to be worth it."

This reverts commit d703fe3a21b09f13d34e5d214afc6fe7c0d3e983.
- Revert "Use mul_add with a float inversion instead of mul_neg_add"
- Revert "Use a branch with less forced inlining"

This reverts commit 979e3577d2b0b3fad3a42e2c53c23384bb6023ca.
- Revert "Revert "Use mul_add with a float inversion instead of mul_neg_add""

This reverts commit 3242122fda638d0571a6f1c99b7f3cdf2a946d16.
- Revert "Failed experiment: use dynamically sized storage instead of stack scratch space; collapses performance"

This reverts commit 6ad19afbfd3c45d85abfae4ce565cd077f276bfe.
- Revert "Another attempt at massaging f64 assembly; didn't work"

This reverts commit a2f7c2ac2821262cd4ef46e6131142f6ec752ebd.

## [0.3.0] - 2025-09-04

### Features

- Add DIT FFT algorithm & bit reversal control

### Bug Fixes

- Bump `bytemuck` to latest version

### Documentation

- Add example usage of interleaved fft

## [0.2.2] - 2025-09-03

### Features

- Add benchmark using criterion
- Add a more robust round-trip FFT test
- Add git hooks and contributing guide
- Add deinterleaving function
- Add criterion group for forward FFT f32
- Add new python benchmarking "framework"
- Add #![feature(doc_cfg)] to make rustdoc actually accept doc_cfg

### Bug Fixes

- Fix formatting
- Fix formatting
- Fix formatting
- Fix formatting
- Fix docsrs config according to Clippy

tbh not sure if Clippy is right in this case
- Avx512 is now stable, so placate clippy
- Update `criterion` benchmarks

### Other

- Vectorize deinterleaving of AoS --> SoA

Use bytemuck + SIMD::deinterleave to rearrange input data from a slice
of Complex values into 2 slices of f32 or f64 values
- Put macro definition begind feature flag
- Account for different signal sizes
- Forgot to add benchmark file
- Don't gate feature documentation on docs.rs, we're on nightly anyway
- Undo duplicate docs.rs all-features

### Refactor

- Make sure benchmark runs
- Make planner reusable

- Planner should be re-usable so it can be re-used for FFT's of the same
  size

- Add regression tests to make sure `fft_64`/`fft_32` gives the same
  results as `fft_64_with_opts_and_plan`/`fft_32_with_opts_and_plan`
- Move planner completely outside of bench function
- Avoid cloning twiddles
- Make examples output time elapsed in nanoseconds
- Use new de-interleaving function
- Make required cfg show up on docs.rs
- Make docs.rs build docs with all features enabled

### Documentation

- Update the normalization section

### Miscellaneous Tasks

- Remove index tracker from hot path
- Updates to benchmarks without planners
- Remove `array_chunks`
- Update pre-commit hook to run clippy everywhere
- Update dependencies and their usage

### Revert

- Revert "Make sure benchmark runs"

This reverts commit 7011dfc040b866cd773185667915a9920b9c5a80.
- Revert "Forgot to add benchmark file"

This reverts commit b70dd4b318f59af32740c4bd4c9e75a69bcd690b.

## [0.2.1] - 2024-05-03

### Features

- Add a function to separate AoS to SoA
- Add reverse separate fn and add test
- Add fft_*_interleaved impls and tests

### Bug Fixes

- Fix formatting
- Fix formatting
- Fix formatting
- Fixes inverse FFT ouput order issue

### Other

- Advertise runtime feature selection
- Transition `num-complex` dependency to optional

- Make num-complex optional and non-default, and all functions that take
  and test interleaved FFT are now under the same feature

- Bump num-complex version 0.4.5 --> 0.4.6

- Enable num-complex for docs/docs.rs

- Update github action workflow to run tests for all features

- Fix formatting in docs to fix links
- Placate clippy's complaints about assign op

### Refactor

- Simplify separate_re_im
- Move twiddles fwd * rev = identity test

- Testing that all the values in the forward twiddle factors pointwise
multiplied by all the values in the inverse twiddle factors is more of a
unit test for twiddles. It made no sense to keep it in the planner.

- The previous commit modifies the planner in a way that this test would
  no longer pass. We can get the same, good coverage by keeping this
  test under twiddles.

### Documentation

- Update docs for interleaved fft

## [0.2.0] - 2024-04-25

### Features

- Add badges for docs and latest published version
- Add more examples and fix typos in readme/plots

- Added rust bindings of FFTW as an example, which will be used for
  benchmarks

- Add fftw (rust bindings) crate as a dev-dependency

- Add an example of using pyphastft to reproduce an example use case of
  FFT from the FFT wikipedia page

- Fix typos in the README and distinguish pyphastft from phastft in the
  python benchmarks plots
- Add todo for improving twiddle generation perf
- Add normalization & output order info. closes #13
- Add tests for 32 bit planner
- Add audio visualization as example for pyphastft
- Add automatic CPU feature detection

### Bug Fixes

- Fix formatting with `cargo fmt`
- Fix formatting
- Fix lint issue brought up by CI

- Fixed `clippy::needless_doctest_main` issue in README

### Other

- Swapped out use of sincos() for more portability across platforms
- Implement SIMDized twiddle generation using macro
- Finish adding test macros for f32/f64

- Make float comparison generic for f32/f64

- Add f32/f64 tests for twiddles using macros

### Refactor

- Make cobra mod public for fast bit reversal
- Use 16 lanes for f32 in SIMD butterfly kernel
- Run `cargo fmt`

### Documentation

- Update README and docs

### Miscellaneous Tasks

- Updated benchmark instructions and fixed typos
- Update python benchmark plot title
- Update benchmark plots
- Update benchmark, plotting, and profiling scripts
- Remove duplicate public functions
- Bump black from 24.1.1 to 24.3.0 in /benches

Bumps [black](https://github.com/psf/black) from 24.1.1 to 24.3.0.
- [Release notes](https://github.com/psf/black/releases)
- [Changelog](https://github.com/psf/black/blob/main/CHANGES.md)
- [Commits](https://github.com/psf/black/compare/24.1.1...24.3.0)

---
updated-dependencies:
- dependency-name: black
  dependency-type: direct:production
...

Signed-off-by: dependabot[bot] <support@github.com>
- Update example and demo video
- Cleanup docs and examples
- Update codecov uploader to v4
- Update benchmarking readme
- Update benchmark plots
- Update pyphastft benchmark plots

## [0.1.1] - 2024-02-13

### Other

- Release v0.1.1 and update name for python wrapper

### Documentation

- Update README.md

Just skip to features, right away

## [0.1.0] - 2024-02-12

### Features

- Add cargo config
- Add rayon back
- Add chunk size 4 kernel
- Add opt flags
- Add check for dups
- Add COBRA initial impl pre-build twiddle factors
- Add basic tests for bit reversal
- Add bash script for profiling and update readme
- Add a sequential bit reversal permutation
- Add iterator for roots of unity
- Add cobra implementation and integrate into FFT
- Add separate mod for kernels
- Add separate kernel for size 8 butterflies
- Add utils
- Add benchmarks for pyfftw, numpy, and PHFT
- Add test for the SIMD impl
- Add github workflow for CI
- Add status badge
- Add tests for bit rev; rand signal; touch-up docs
- Add codecov job and codecov config for 90% target
- Add code coverage badge
- Add benchmarking scripts
- Add dir with instructions & code to run benchmarks
- Add FFTW3 benchmark
- Add Options struct
- Add fftw3 to benchmark script
- Add a public function that accepts caller-provided `Options`
- Add benchmark plots
- Add plots to readme
- Add `Planner` and ammend examples, tests, etc.
- Add planner mod
- Add bench system config info
- Add license files
- Add asert to check input length is 2^n

- Add a regression test to make sure non-power-of-two FFTs are not
  allowed for the time being
- Add instructions for reproducing benchmarks
- Add tests and make API breaking changes

- Add a test to make sure inverse twiddle factors multiplied by forward
  twiddle factors always gives 1.0

- `fft` now takes the real/imaginary input and the `Direction` of the `fft`

- Pre-built planners can be used with the `fft_with_opts_and_plan`
  function
- Add regression test for planner/fft mismatch
- Add regression docs to explain regression test
- Add requirements.txt for benchmarks

### Bug Fixes

- Fixes for profiling
- Fix for prebuilt twiddles and portable SIMD
- Fix assertion
- Fix overhead from bit reversal threading
- Fix typo in benchmark logging
- Fix build
- Fix typos in benchmarking code
- Fix bug in phast benchmark & add fft direction
- Fix FFTW benchmark to use wall time
- Fix perf regression for small input sizes

- Increased block size increases performance for larger input sizes, but
  the increase caused a regression on a AMD 7950x machine, for smaller
  input sizes. The `Planner` needs to be updated to computes an ideal
  block width. In the interim, we will use 2 * CACHE_LINE_SIZE. On
  x86-64, this is usually 2 * 64 = 128.
- Fix assertion in `fft_with_opts_and_plan`

- Twiddles should always be half the size of input
- Fix typo

### Other

- Don't precompute twiddle factors for large chunks
- Split into two binaries for profiling
- Parallelize bit reverse permutation with 2 threads
- Experiments
- Pre-compute twiddle factors for chunk n
- Bug fix -- clear twiddle factor cache after use
- Abstract out twiddle factor generation
- Prebuild only half the required twiddles
- Try out generating twiddles lazily
- Restructure into lib, add examples, scripts, etc.
- Proof of concept of caching 1/4 of twiddle factors
- Unroll for half of twiddle factors
- Reuse generated twiddle factors by filtering
- Go back to only generating half of twiddles
- Faster twiddles hopefully workable concept this time
- Plug SIMD twiddle generation in
- Generate random signal for py benchmarks
- Increase `BLOCK_SIZE` to 256
- Rewrite README
- Tweaks to "Features" in README
- Wire up guessing the options to the main FFT function
- Placate Clippy
- Exponentially decrease # of iters in benchmark
- Change plot ordering so colors are consistent
- Print human readable bytes without decimal
- Adjust pybindings to work with new API
- Expand on the Python installation instructions
- Mention the license in the README
- Set repository and unset documentation field; it will default to docs.rs
- Mention lower memory use in the "vs" section
- Automate findings of latest benchmark results
- Format benchmark scripts
- Expose `cobra_apply` for fast bit reverseal

### Refactor

- Use std `reverse_bits` for performance improvement
- Use faster impl of bit reversal implementation
- Replace vec with array since it's < 512_000 bytes
- Switch back to pre-built cache using vec
- Use in-place bit reversal in COBRA for small N
- Move kernels to separate mod
- Simplify size 2 butterfly
- Use numpy crate to avoid overhead in pybindings
- Use scipy to generate random signal
- Move rustfft benchmark to examples
- Use random signal for benchmarking rustfft
- Use random signal generator to benchmark phastft
- Move utility functions to separate lib
- Simplify options down to a single multi-threading knob

### Documentation

- Update README
- Update README with basic skeleton
- Update READMEs
- Update README
- Update README
- Update README text to match latest benchmarks
- Update documentation and readme
- Update README.md
- Update README.md

### Performance

- Optimize twiddles pre-compute
- Improve recursive bit-reversal perf for testing

- Tests run much faster now with a bit of loop unrolling, loop fusion,
  and pre-allocating vectors with the right sizes

- Add documentation and cite references
- Improve assertion functions

### Miscellaneous Tasks

- Remove rayon for profiling
- Cleanup for profiling
- Remove rayon for profiling
- Remove unused mods
- Update for benchmarks
- Update profiling section in readme
- Update profile script, add profiling to examples
- Remove useless files
- Remove benchmark mod from lib
- Cleanup all warnings and unused code
- Remove rayon dep, add bar plot, rename to PhastFT
- Update workflow
- Remove old BRAVO implementation written in python
- Updates to examples and plotting script
- Update benchmark plotting
- Update benchmark scripts
- Remove workspace
- Cleanup x-axis of plots
- Update plots for latest benchmarks
- Update the license in Cargo.toml
- Drop unused author fields
- Update MIT license for future contributors
- Cleanup plots and split bar plot for py benches
- Update benchmark plots
- Update saved figure names in benches
- Update benchmark readme

### Revert

- Revert to creating twiddle factors for large n
- Revert "Prebuild only half the required twiddles"

This reverts commit e552fb1ea2b5202c47e4cf1e8392adcefe1d476b.
- Revert "Proof of concept of caching 1/4 of twiddle factors"

This reverts commit de2cca02d99c93b76e3b42bf09d29deeb2eda62f.
- Revert "Add separate kernel for size 8 butterflies"

This reverts commit 249be77548a8b2f23aca4739a5d3aad536939cba.

<!-- generated by git-cliff -->
