# rustsimdfun

Some rust SIMD learning.


## Monday, 15th Sept 2025

### average_float - using rust SIMD intrinsics

Compute the average of a large array of f32 using SIMD if available.
Supports AVX-512 on x86_64 and NEON on aarch64.

Detects AVX-512f at runtime too.

```shell
cargo run --release --bin average_float
```

Output on a Macbook Air M4

```
average_serial = 0.200820, average = 0.199793, diff = 0.001028
serial: v = 0.200820, elapsed = 0.563 ms
serial: v = 0.200820, elapsed = 0.569 ms
serial: v = 0.200820, elapsed = 0.594 ms
serial: v = 0.200820, elapsed = 0.588 ms
serial: v = 0.200820, elapsed = 0.590 ms
simd  : v = 0.199793, elapsed = 0.143 ms
simd  : v = 0.199793, elapsed = 0.143 ms
simd  : v = 0.199793, elapsed = 0.134 ms
simd  : v = 0.199793, elapsed = 0.139 ms
simd  : v = 0.199793, elapsed = 0.137 ms
avg times: serial = 0.581 ms, simd = 0.139 ms, speedup = 4.17x
```

