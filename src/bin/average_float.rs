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
