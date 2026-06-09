# Bit-Reversal Kernel Benchmarks

A head-to-head micro-benchmark of five bit-reversal *permutation* algorithms<p align="center">
  <img src="https://raw.githubusercontent.com/smu160/PhastFT/main/assets/criterion_overlay_kernel_bit_reversal_f32_10_16.svg" width="400" title="Bit Reversal (f32), small-N" alt="Bit Reversal (f32), 2^10–2^16: COBRAVO, BRAVO, COBRA, Elaan, Naive BR runtime relative to COBRA">
  <img src="https://raw.githubusercontent.com/smu160/PhastFT/main/assets/criterion_overlay_kernel_bit_reversal_f32_17_24.svg" width="400" title="Bit Reversal (f32), large-N" alt="Bit Reversal (f32), 2^17–2^24: COBRAVO, BRAVO, COBRA, Elaan, Naive BR runtime relative to COBRA">
</p>

<p align="center">
  <img src="https://raw.githubusercontent.com/smu160/PhastFT/main/assets/criterion_overlay_kernel_bit_reversal_f64_10_16.svg" width="400" title="Bit Reversal (f64), small-N" alt="Bit Reversal (f64), 2^10–2^16: COBRAVO, BRAVO, COBRA, Elaan, Naive BR runtime relative to COBRA">
  <img src="https://raw.githubusercontent.com/smu160/PhastFT/main/assets/criterion_overlay_kernel_bit_reversal_f64_17_24.svg" width="400" title="Bit Reversal (f64), large-N" alt="Bit Reversal (f64), 2^17–2^24: COBRAVO, BRAVO, COBRA, Elaan, Naive BR runtime relative to COBRA">
</p>

## Steps to Reproduce

From the repo root, run the following:
```sh
cargo bench --bench bit_reversal --features bench-internals
```

# Regenerate these overlays (two SVGs per precision) into the current directory.
```sh
uv run benches/plot_criterion_overlay.py --groups kernel_bit_reversal_f32,kernel_bit_reversal_f64
```
