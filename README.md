# rustsimdfun

Some rust SIMD learning.


## Monday, 15th Sept 2025

Hey hey.
For me, I'm just learning about rust and SIMD things, and trying out a few ideas.
I was thinking about some things a few weeks ago... https://fosstodon.org/@renedudfield/115056704976306365

Here's a few links I've been reading.

- maybe best portable SIMD library C++https://github.com/google/highway
- rust portable SIMD API https://github.com/rust-lang/portable-simd and https://doc.rust-lang.org/std/simd/index.html
  - learning rust blog post https://levelup.gitconnected.com/learning-rust-simd-3305e576b1ab
  - https://crates.io/crates/packed_simd (prototype nightly implementation of portable simd)
  - packed_simd examples: https://github.com/rust-lang/packed_simd/tree/master/examples
  - coresimd/examples: https://github.com/rust-lang/portable-simd/tree/master/crates/core_simd/examples
- https://halide-lang.org/
- https://github.com/DLTcollab/sse2neon
- Dynamic runtime assembler for rust https://github.com/CensoredUsername/dynasm-rs


I started with using rust intrinsics that are built into rust now. 
It conditionally builds different functions depending on the target.
For example, on intel it builds avx512 binary, but at runtime it detects if avx512 is enabled.
On arm64 target with neon it builds a using neon intrinsics.

Also there's a basic correctness and benchmarking comparisons of the serial version of the function to the simd one.

I'll try making different versions of this functinons with different techniques. For example a runtime JIT compiled version,
and a version with portable SIMD.


### average_float - using rust SIMD intrinsics

See [bin/average_float.rs](https://github.com/illume/rustsimdfun/blob/main/src/bin/average_float.rs) 
for the way SIMD intrinsics are done in rust today.

Compute the average of a large array of f32 using SIMD if available.
Supports AVX-512 on x86_64 and NEON on aarch64.

Detects AVX-512f at runtime too.

```shell
cargo run --release --bin average_float
```


You need to enable avx512 feature:

```shell
cargo run --release --bin average_float --features avx512
```

#### Macbook Air M4 - 4x speedup with neon

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

### Intel Surface book 3 - 10x speedup with avx512

```
average_serial = 0.200820, average = 0.200052, diff = 0.000769
serial: v = 0.200820, elapsed = 0.685 ms
serial: v = 0.200820, elapsed = 0.638 ms
serial: v = 0.200820, elapsed = 0.793 ms
serial: v = 0.200820, elapsed = 0.690 ms
serial: v = 0.200820, elapsed = 0.709 ms
simd  : v = 0.200052, elapsed = 0.113 ms
simd  : v = 0.200052, elapsed = 0.053 ms
simd  : v = 0.200052, elapsed = 0.057 ms
simd  : v = 0.200052, elapsed = 0.064 ms
simd  : v = 0.200052, elapsed = 0.044 ms
avg times: serial = 0.703 ms, simd = 0.066 ms, speedup = 10.65x
```


