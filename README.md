# wgpu-bench
[![Build Status](https://github.com/kvark/wgpu-bench/workflows/CI/badge.svg?branch=master)](https://github.com/kvark/wgpu-bench/actions)

Benchmark of WebGPU native implementation in Rust - [wgpu-rs](https://github.com/gfx-rs/wgpu-rs).

Areas:
  - CPU overhead, e.g. creating resources and recording commands
  - CPU memory and descriptor allocator efficiency
  - GPU operation cost, e.g. fill rate
