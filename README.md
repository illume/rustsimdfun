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

## Tuesday, 16th Sept 2025

I read up on wide, and pulp today. Super interesting! I keep uncovering more SIMD in Rust stuff I haven't encountered yet.

Another improvement on Rust SIMD in the last several months is that AVX-512 support is in stable Rust now. With this I'm seeing faster results on an old 2019 intel CPU than an m4 chip (which is in general lots faster).

I was reading through the portable-simd repo and [I also learnt something new](https://github.com/rust-lang/portable-simd/pull/420/files) about how Glibc can load instruction specific libraries itself now!
> * Newer distros (with Glibc 2.33 or later) support `glibc-hwcaps` directory that allow you to place optimized libraries. Instead of putting the libraries in the normal locations, you can create `glibc-hwcaps` directory at those locations, and it can have subdirectories such as `x86-64-v2` (supports up to `sse4.2`), `x86-64-v3` (supports up to `avx2`), and `x86-64-v4` (supports `avx512f`, `avx512bw`, `avx512cd`, `avx512dq`, and `avx512vl`) where you can place the libraries built with those target CPUs. When loading a library, Glibc will automatically load the most optimized library supported by the CPU.

### dynasmrt

This is a runtime assembler, similar to the one used by Luajit (or Softwire as was or is used in Chrome for graphics compilation).

https://censoredusername.github.io/dynasm-rs/language/tutorial.html


```rust
use dynasmrt::{dynasm, DynasmApi, DynasmLabelApi};

use std::{io, slice, mem};
use std::io::Write;

fn main() {
    let mut ops = dynasmrt::x64::Assembler::new().unwrap();
    let string = "Hello World!";

    dynasm!(ops
        ; .arch x64
        ; ->hello:
        ; .bytes string.as_bytes()
    );

    let hello = ops.offset();
    dynasm!(ops
        ; .arch x64
        ; lea rcx, [->hello]
        ; xor edx, edx
        ; mov dl, BYTE string.len() as _
        ; mov rax, QWORD print as _
        ; sub rsp, BYTE 0x28
        ; call rax
        ; add rsp, BYTE 0x28
        ; ret
    );

    let buf = ops.finalize().unwrap();

    let hello_fn: extern "win64" fn() -> bool = unsafe { mem::transmute(buf.ptr(hello)) };

    assert!(hello_fn());
}

pub extern "win64" fn print(buffer: *const u8, length: u64) -> bool {
    io::stdout()
        .write_all(unsafe { slice::from_raw_parts(buffer, length as usize) })
        .is_ok()
}
```



I tried... but couldn't get it to work. Perhaps because my neon experience isn't very good,
but I kept getting errors and eventually gave up.


## Wednesday, 17th Sept 2025

I tried a bit with dynasmrt some more, but no luck making a neon version of the function
for average_float. Still, was quite interesting.

Had a bunch of chats with Magnus about SIMD. One was about how to combine multiple requests 
for the same function. If each request does various small amounts of work, maybe SIMD doesn't
make sense so much. But if you rewrite the function to handle 100s of request at once then
you can batch the work and SIMD functions may be useful in this case. For example, 
decoding 1000 small base64 strings at once, doing 1 DB select rather than 1000. 


### Portable SIMD average_float_portable_simd

Portable simd is a future standard allowing you to write portable SIMD code
that works on different architectures.

It's still only in nightly.

```shell
rustup update -- nightly
rustup override set nightly
```

Here's the portable simd version.

```rust
/// Compute the average of a slice of f32s using portable SIMD (8 lanes).
pub fn average_float_portable_simd(data: &[f32]) -> f32 {
    use std::simd::Simd;
    use std::simd::prelude::SimdFloat;
    // 8 lanes of f32 in one SIMD vector
    type Vf32 = Simd<f32, 8>;
    const LANES: usize = 8;

    let len = data.len();
    if len == 0 {
        return 0.0;
    }

    // Accumulate in SIMD
    let mut sum = Vf32::splat(0.0);
    let chunks = len / LANES;
    for i in 0..chunks {
        let start = i * LANES;
        let v = Vf32::from_slice(&data[start..start + LANES]);
        sum += v;
    }

    // Horizontal reduction
    let mut total = sum.reduce_sum();

    // Handle remainder
    for &x in &data[chunks * LANES..] {
        total += x;
    }

    total / (len as f32)
}
```

#### Macbook air M4, Portable SIMD 9.4x faster than serial, 2.25x faster than manual version

So How fast is in on Macbook air M4?

``` rust
average_serial = 0.200820, average = 0.199793, average_portable = 0.200124
diffs: serial-simd = 0.001028, serial-portable = 0.000697, simd-portable = 0.000331
serial:          v = 0.200820, elapsed = 0.594 ms
serial:          v = 0.200820, elapsed = 0.591 ms
serial:          v = 0.200820, elapsed = 0.580 ms
serial:          v = 0.200820, elapsed = 0.581 ms
serial:          v = 0.200820, elapsed = 0.585 ms
simd:            v = 0.199793, elapsed = 0.140 ms
simd:            v = 0.199793, elapsed = 0.154 ms
simd:            v = 0.199793, elapsed = 0.139 ms
simd:            v = 0.199793, elapsed = 0.135 ms
simd:            v = 0.199793, elapsed = 0.146 ms
portable simd:   v = 0.200124, elapsed = 0.074 ms
portable simd:   v = 0.200124, elapsed = 0.075 ms
portable simd:   v = 0.200124, elapsed = 0.074 ms
portable simd:   v = 0.200124, elapsed = 0.074 ms
portable simd:   v = 0.200124, elapsed = 0.075 ms
avg times: serial = 0.586 ms, simd = 0.143 ms, portable simd = 0.074 ms
speedups:  simd = 4.10x, portable simd = 7.89x, simd/portable = 1.92x
```

Why is it faster? I didn't look into it, but I guess it's using more lanes.


I tried changing the lanes from 8 to 16, and on the mac m4 it goes a bit quicker.

```rust
average_serial = 0.200820, average = 0.199793, average_portable = 0.100026
diffs: serial-simd = 0.001028, serial-portable = 0.100795, simd-portable = 0.099767
serial:          v = 0.200820, elapsed = 0.577 ms
serial:          v = 0.200820, elapsed = 0.582 ms
serial:          v = 0.200820, elapsed = 0.576 ms
serial:          v = 0.200820, elapsed = 0.582 ms
serial:          v = 0.200820, elapsed = 0.589 ms
simd:            v = 0.199793, elapsed = 0.157 ms
simd:            v = 0.199793, elapsed = 0.134 ms
simd:            v = 0.199793, elapsed = 0.139 ms
simd:            v = 0.199793, elapsed = 0.135 ms
simd:            v = 0.199793, elapsed = 0.131 ms
portable simd:   v = 0.100026, elapsed = 0.064 ms
portable simd:   v = 0.100026, elapsed = 0.062 ms
portable simd:   v = 0.100026, elapsed = 0.062 ms
portable simd:   v = 0.100026, elapsed = 0.062 ms
portable simd:   v = 0.100026, elapsed = 0.061 ms
avg times: serial = 0.581 ms, simd = 0.139 ms, portable simd = 0.062 ms
speedups:  simd = 4.18x, portable simd = 9.39x, simd/portable = 2.25x
```

This shows one of the benefits of being able to change lanes so easily.
Instead of rewriting the intrinsics, I changed an 8 to 16 and recompiled.

#### Surface book 3, Portable SIMD 7.9x faster than serial, 1.16x faster than manual avx version

```rust
average_serial = 0.200820, average = 0.200052, average_portable = 0.200124
diffs: serial-simd = 0.000769, serial-portable = 0.000697, simd-portable = 0.000072
serial:          v = 0.200820, elapsed = 0.645 ms
serial:          v = 0.200820, elapsed = 0.607 ms
serial:          v = 0.200820, elapsed = 0.611 ms
serial:          v = 0.200820, elapsed = 0.611 ms
serial:          v = 0.200820, elapsed = 0.731 ms
simd:            v = 0.200052, elapsed = 0.097 ms
simd:            v = 0.200052, elapsed = 0.063 ms
simd:            v = 0.200052, elapsed = 0.046 ms
simd:            v = 0.200052, elapsed = 0.039 ms
simd:            v = 0.200052, elapsed = 0.038 ms
portable simd:   v = 0.200124, elapsed = 0.076 ms
portable simd:   v = 0.200124, elapsed = 0.085 ms
portable simd:   v = 0.200124, elapsed = 0.097 ms
portable simd:   v = 0.200124, elapsed = 0.103 ms
portable simd:   v = 0.200124, elapsed = 0.112 ms
avg times: serial = 0.641 ms, simd = 0.057 ms, portable simd = 0.094 ms
speedups:  simd = 11.32x, portable simd = 6.79x, simd/portable = 0.60x
```


With 16 lanes it's faster than my manually written avx512 version.

```rust
average_serial = 0.200820, average = 0.200052, average_portable = 0.100026
diffs: serial-simd = 0.000769, serial-portable = 0.100795, simd-portable = 0.100026
serial:          v = 0.200820, elapsed = 0.647 ms
serial:          v = 0.200820, elapsed = 0.645 ms
serial:          v = 0.200820, elapsed = 0.621 ms
serial:          v = 0.200820, elapsed = 0.703 ms
serial:          v = 0.200820, elapsed = 0.714 ms
simd:            v = 0.200052, elapsed = 0.154 ms
simd:            v = 0.200052, elapsed = 0.071 ms
simd:            v = 0.200052, elapsed = 0.141 ms
simd:            v = 0.200052, elapsed = 0.070 ms
simd:            v = 0.200052, elapsed = 0.053 ms
portable simd:   v = 0.100026, elapsed = 0.088 ms
portable simd:   v = 0.100026, elapsed = 0.104 ms
portable simd:   v = 0.100026, elapsed = 0.057 ms
portable simd:   v = 0.100026, elapsed = 0.076 ms
portable simd:   v = 0.100026, elapsed = 0.097 ms
avg times: serial = 0.666 ms, simd = 0.098 ms, portable simd = 0.084 ms
speedups:  simd = 6.81x, portable simd = 7.90x, simd/portable = 1.16x
```

