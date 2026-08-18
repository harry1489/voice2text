use std::path::Path;

use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

fn main() {
    let wav_path = std::env::args().nth(1).expect("usage: transcribe <file.wav>");
    let model_path = std::env::var("V2T_MODEL")
        .unwrap_or_else(|_| "/home/harry/copilot/models/ggml-small.en.bin".into());

    let mut cparams = WhisperContextParameters::default();
    cparams.use_gpu = false;
    let ctx = WhisperContext::new_with_params(&model_path, cparams).expect("load model");

    let bytes = std::fs::read(&wav_path).expect("read wav");
    let channels = u16::from_le_bytes([bytes[22], bytes[23]]) as usize;
    let rate = u32::from_le_bytes([bytes[24], bytes[25], bytes[26], bytes[27]]);
    let bits = u16::from_le_bytes([bytes[34], bytes[35]]);
    assert_eq!(bits, 16, "expected 16-bit pcm");

    let mut data = 12;
    while data + 8 <= bytes.len() {
        let size = u32::from_le_bytes([bytes[data + 4], bytes[data + 5], bytes[data + 6], bytes[data + 7]]) as usize;
        if &bytes[data..data + 4] == b"data" {
            data += 8;
            break;
        }
        data += 8 + size + (size % 2);
    }

    let data = &bytes[data..];
    let mut samples = Vec::with_capacity(data.len() / 2);
    for frame in data.chunks_exact(2) {
        let sample = i16::from_le_bytes([frame[0], frame[1]]) as f32 / i16::MAX as f32;
        samples.push(sample);
    }
    if channels > 1 {
        let ch = channels;
        let mut mono = Vec::with_capacity(samples.len() / ch);
        for f in samples.chunks_exact(ch) {
            mono.push(f.iter().sum::<f32>() / ch as f32);
        }
        samples = mono;
    }
    if rate != 16_000 {
        let ratio = rate as f64 / 16_000.0;
        let cutoff = if ratio >= 1.0 { 0.5 / ratio } else { 0.5 } * 0.95;
        const TAPS: f64 = 32.0;
        let out_len = (samples.len() as f64 / ratio).floor() as usize;
        let mut resampled = Vec::with_capacity(out_len);
        for i in 0..out_len {
            let center = i as f64 * ratio;
            let start = ((center - TAPS).ceil() as isize).max(0);
            let end = ((center + TAPS).floor() as isize).min(samples.len() as isize - 1);
            let mut sum = 0.0;
            let mut weight_sum = 0.0;
            let mut j = start;
            while j <= end {
                let x = center - j as f64;
                let arg = 2.0 * cutoff * x;
                let s = if arg == 0.0 {
                    1.0
                } else {
                    (std::f64::consts::PI * arg).sin() / (std::f64::consts::PI * arg)
                };
                let w = 2.0
                    * cutoff
                    * s
                    * (0.5 + 0.5 * (std::f64::consts::PI * x / TAPS).cos());
                sum += samples[j as usize] as f64 * w;
                weight_sum += w;
                j += 1;
            }
            resampled.push((sum / weight_sum.max(1e-9)) as f32);
        }
        samples = resampled;
    }

    eprintln!("[transcribe] {}: {} samples @16k ({:.1}s)", Path::new(&wav_path).file_name().unwrap().to_string_lossy(), samples.len(), samples.len() as f64 / 16000.0);

    let mut state = ctx.create_state().expect("create state");
    let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
    params.set_n_threads(8);
    params.set_language(Some("en"));
    params.set_no_timestamps(true);
    params.set_single_segment(true);
    params.set_suppress_blank(true);
    params.set_suppress_nst(true);
    params.set_print_special(false);
    params.set_print_progress(false);
    params.set_print_realtime(false);
    params.set_no_speech_thold(0.6);

    state.full(params, &samples).expect("run whisper");

    let mut out = String::new();
    for seg in state.as_iter() {
        if seg.no_speech_probability() < 0.6 {
            if let Ok(text) = seg.to_str() {
                out.push_str(text);
            }
        }
    }
    println!("{}", out.trim());
}
