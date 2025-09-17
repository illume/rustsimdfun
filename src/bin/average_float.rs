// average_float.rs
// Compute the average of a large array of f32 using SIMD if available.
// Supports AVX-512 on x86_64 and NEON on aarch64.

fn average_serial(data: &[f32]) -> f32 {
    let sum: f32 = data.iter().copied().sum();
    sum / data.len() as f32
}

pub fn average(data: &[f32]) -> f32 {
    #[cfg(all(feature = "avx512", target_arch = "x86_64"))]
    {
        if std::is_x86_feature_detected!("avx512f") {
            return unsafe { average_avx512(data) };
        }
    }

    #[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
    {
        return unsafe { average_neon(data) };
        // return unsafe { average_dynasm_neon(data) };
    }

    #[allow(unreachable_code)]
    average_serial(data)
}

#[cfg(all(feature = "avx512", target_arch = "x86_64"))]
#[target_feature(enable = "avx512f")]
unsafe fn average_avx512(data: &[f32]) -> f32 {
    use std::arch::x86_64::*;
    let ptr = data.as_ptr();
    let chunks = data.len() / 16;
    let mut acc = _mm512_setzero_ps();

    for i in 0..chunks {
        let v = _mm512_loadu_ps(ptr.add(i * 16));
        acc = _mm512_add_ps(acc, v);
    }
    let mut sum = _mm512_reduce_add_ps(acc);
    for i in (chunks * 16)..data.len() {
        sum += unsafe { *ptr.add(i) };
    }
    sum / (data.len() as f32)
}

#[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
#[target_feature(enable = "neon")]
unsafe fn average_neon(data: &[f32]) -> f32 {
    use std::arch::aarch64::*;
    let ptr = data.as_ptr();
    let chunks = data.len() / 4;
    let mut acc = vdupq_n_f32(0.0);
    for i in 0..chunks {
        let v = unsafe { vld1q_f32(ptr.add(i * 4)) };
        acc = vaddq_f32(acc, v);
    }
    let mut sum = vaddvq_f32(acc);
    for i in (chunks * 4)..data.len() {
        sum += unsafe { *ptr.add(i) };
    }
    sum / (data.len() as f32)
}

// Alternative NEON implementation using dynasm for JIT assembly generation.
// I couldn't get this to work yet.
// https://censoredusername.github.io/dynasm-rs/language/tutorial.html
// See aarch64 reference https://censoredusername.github.io/dynasm-rs/language/langref_aarch64.html
// See aarch64 instruction set https://censoredusername.github.io/dynasm-rs/language/instructionref_aarch64.html
// See test examples for code https://github.com/CensoredUsername/dynasm-rs/blob/master/testing/tests/gen_aarch64/aarch64_tests_1.rs.gen
//
// #[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
// #[target_feature(enable = "neon")]
// unsafe fn average_dynasm_neon(data: &[f32]) -> f32 {
//     use dynasmrt::{dynasm, DynasmApi, ExecutableBuffer};
//     use std::mem;

//     let ptr = data.as_ptr();
//     let chunks = data.len() / 4;
//     let mut out = [0f32; 4]; // will receive v0 lanes from JIT

//     let mut ops = dynasmrt::aarch64::Assembler::new().unwrap();
//     let entry_point = ops.offset();
//     dynasm!(ops
//         ; .arch aarch64
//         // ; // x0 = data ptr, x1 = chunks, x2 = out ptr
//         ; cbz x1, =>loop_end
//         // ; // zero v0
//         ; dup V0.S4, wzr
//         ;->loop_start:
//         ; ld1 {v1.4s}, [x0], 16
//         ; fadd v0.4s, v0.4s, v1.4s
//         ; subs x1, x1, 1
//         ; bne =>loop_start
//         ;->loop_end:
//         ; st1 {v0.4s}, [x2]
//         ; ret
//     );
//     let buf: ExecutableBuffer = ops.finalize().unwrap();
//     let func: extern "C" fn(*const f32, usize, *mut f32) =
//         mem::transmute(buf.ptr(entry_point));

//     func(ptr, chunks, out.as_mut_ptr());

//     // reduce the 4-lane accumulator stored by the JIT
//     let mut sum: f32 = out.iter().copied().sum();

//     // handle remaining tail scalars safely
//     for i in (chunks * 4)..data.len() {
//         sum += *ptr.add(i);
//     }

//     sum / (data.len() as f32)
// }

fn init_data(len: usize) -> Vec<f32> {
    vec![0.2_f32; len]
}

fn main() {
    const N: usize = 524_288;
    let data = init_data(N);

    // correctness
    let s_serial = average_serial(&data);
    let s_simd = average(&data);
    println!(
        "average_serial = {:.6}, average = {:.6}, diff = {:.6}",
        s_serial,
        s_simd,
        (s_serial - s_simd).abs()
    );

    // benchmark
    use std::time::Instant;
    const RUNS: usize = 5;

    // warmup
    let _ = average(&data);
    let _ = average_serial(&data);

    // bench serial
    let mut serial_total_ms = 0.0;
    for _ in 0..RUNS {
        let start = Instant::now();
        let v = average_serial(&data);
        let dur = start.elapsed();
        let ms = dur.as_secs_f64() * 1e3;
        println!("serial: v = {:.6}, elapsed = {:.3} ms", v, ms);
        serial_total_ms += ms;
    }

    // bench simd/auto
    let mut simd_total_ms = 0.0;
    for _ in 0..RUNS {
        let start = Instant::now();
        let v = average(&data);
        let dur = start.elapsed();
        let ms = dur.as_secs_f64() * 1e3;
        println!("simd  : v = {:.6}, elapsed = {:.3} ms", v, ms);
        simd_total_ms += ms;
    }

    let serial_avg = serial_total_ms / RUNS as f64;
    let simd_avg = simd_total_ms / RUNS as f64;
    let speedup = if simd_avg > 0.0 {
        serial_avg / simd_avg
    } else {
        0.0
    };
    println!(
        "avg times: serial = {:.3} ms, simd = {:.3} ms, speedup = {:.2}x",
        serial_avg, simd_avg, speedup
    );
}
