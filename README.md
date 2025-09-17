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



I tried... but couldn't get it to work.





