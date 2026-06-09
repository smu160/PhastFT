# Bit-Reversal Kernel Benchmarks

A head-to-head micro-benchmark of five bit-reversal *permutation* algorithms — the
index shuffle a radix-2 Cooley–Tukey FFT performs so its butterflies read and write
in natural order. Each kernel is timed in isolation (no surrounding FFT) over
power-of-two lengths from 2¹⁰ to 2²⁴, in both `f32` and `f64`, with
[criterion](https://crates.io/crates/criterion). Every bar is normalized to the
scalar **COBRA** baseline, so a height of `0.25` means "a quarter of COBRA's time —
4× faster."

## The contenders

- **COBRAVO** — PhastFT's production kernel, and the only one the library actually
  ships. *Cache-Optimal Bit-Reversal Algorithm using Vector permute Operations*:
  SIMD vector-interleave reversal
  ([Lokhmotov & Mycroft, SPAA '07](https://dl.acm.org/doi/abs/10.1145/1248377.1248411))
  with an added cache-tiling pass.
- **BRAVO** — the same SIMD vector-permute reversal **without** COBRAVO's cache
  tiling (PhastFT's earlier, pre-tiling kernel). Here purely to isolate what the
  tiling buys.
- **COBRA** — scalar cache-optimal **blocked** bit reversal
  ([Carter & Gatlin, FOCS '98](https://doi.org/10.1109/SFCS.1998.743505); Knauth et
  al.). The chart baseline; below 2¹⁵ it falls back to Elaan, so the two coincide
  there.
- **Elaan** — in-place, single-buffer Gray-code reversal (a.k.a. "Jennifer's
  method", [katjaas.nl](https://www.katjaas.nl/bitreversal/bitreversal.html)): walks
  an index and its bit-reversed mirror in lockstep, permuting the array in N/4
  iterations with no allocation. Scalar.
- **Naive BR** — the textbook incremental bit-reversed-counter swap loop. The "why
  bother" reference point: tiny, obviously correct, and not remotely cache-friendly.

Sources: COBRAVO in [`../src/algorithms/bravo.rs`](../src/algorithms/bravo.rs); the
un-tiled BRAVO inlined in [`bit_reversal.rs`](bit_reversal.rs); COBRA, Elaan, and
Naive BR in [`legacy_bitrev.rs`](legacy_bitrev.rs).

## Results

<p align="center">
  <img src="https://raw.githubusercontent.com/smu160/PhastFT/main/assets/criterion_overlay_kernel_bit_reversal_f32_10_16.svg" width="400" title="Bit Reversal (f32), small-N" alt="Bit Reversal (f32), 2^10–2^16: COBRAVO, BRAVO, COBRA, Elaan, Naive BR runtime relative to COBRA">
  <img src="https://raw.githubusercontent.com/smu160/PhastFT/main/assets/criterion_overlay_kernel_bit_reversal_f32_17_24.svg" width="400" title="Bit Reversal (f32), large-N" alt="Bit Reversal (f32), 2^17–2^24: COBRAVO, BRAVO, COBRA, Elaan, Naive BR runtime relative to COBRA">
</p>

<p align="center">
  <img src="https://raw.githubusercontent.com/smu160/PhastFT/main/assets/criterion_overlay_kernel_bit_reversal_f64_10_16.svg" width="400" title="Bit Reversal (f64), small-N" alt="Bit Reversal (f64), 2^10–2^16: COBRAVO, BRAVO, COBRA, Elaan, Naive BR runtime relative to COBRA">
  <img src="https://raw.githubusercontent.com/smu160/PhastFT/main/assets/criterion_overlay_kernel_bit_reversal_f64_17_24.svg" width="400" title="Bit Reversal (f64), large-N" alt="Bit Reversal (f64), 2^17–2^24: COBRAVO, BRAVO, COBRA, Elaan, Naive BR runtime relative to COBRA">
</p>

Each bar is a series' **median runtime ÷ COBRA's median** at that size; whiskers are
the inter-quartile range; the dashed line at `1.0` is COBRA by construction. **Lower
is faster.** The x-axis is input size in bytes; the 2¹⁰–2²⁴ sweep is split into a
small-N half (2¹⁰–2¹⁶) and a large-N half (2¹⁷–2²⁴).

**What the numbers say.** COBRAVO is the fastest kernel across nearly the entire
range, but *where* it wins is the interesting part. While the array still fits in
cache (≲2¹⁴), the two SIMD reversals are indistinguishable — COBRAVO ≈ BRAVO at
~0.27× COBRA for `f32` (~0.4× for `f64`), roughly 3–4× faster than scalar COBRA and
Elaan (a bit less for `f64`) and 8–12× faster than the naive loop — because
COBRAVO's tiling hasn't engaged yet. Once the data spills cache (≈2¹⁶ and up) the
tiling earns its "CO": at 2²⁰ COBRAVO drops to ~0.15× COBRA for `f32` (≈6–7× faster)
and runs ~3× faster than the *un-tiled* BRAVO. At the largest sizes (2²³–2²⁴) memory
bandwidth dominates and the field compresses; for `f64` the un-tiled BRAVO even edges
past COBRAVO (0.38× vs 0.51× at 2²⁴) — within the noise of these few-sample,
multi-millisecond runs. Finally, the dramatic dip at 2¹⁵ is a COBRA artifact, not a
universal speedup spike: COBRA's scalar blocked path switches on exactly there with a
sharp cost jump (below 2¹⁵ it simply *is* Elaan), so the baseline is anomalously slow
at that one size.

## Reproduce

Run both commands from the **repo root**:

```bash
# Run the kernel benchmark (writes target/criterion/kernel_bit_reversal_f{32,64}/).
cargo bench --bench bit_reversal --features bench-internals

# Regenerate these overlays (two SVGs per precision) into the current directory.
uv run benches/plot_criterion_overlay.py \
    --groups kernel_bit_reversal_f32,kernel_bit_reversal_f64
```

See [`README.md`](README.md) for the full criterion + overlay-plot workflow, the
group-naming conventions, and the other bench targets.
