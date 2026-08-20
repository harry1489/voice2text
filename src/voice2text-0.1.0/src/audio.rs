use std::sync::Mutex;
use std::sync::{Arc, Mutex as StdMutex};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, FromSample, Sample, SampleFormat, SizedSample, SupportedStreamConfig};

const TARGET_RATE: u32 = 16_000;

struct Session {
    stream: cpal::Stream,
    samples: Arc<StdMutex<Vec<f32>>>,
    sample_rate: u32,
    channels: u16,
}

static SESSION: Mutex<Option<Session>> = Mutex::new(None);

pub fn start_recording() -> Result<(), String> {
    if SESSION.lock().unwrap().is_some() {
        return Err("already recording".into());
    }

    let host = cpal::default_host();
    let device = host
        .default_input_device()
        .ok_or_else(|| "no default input device".to_string())?;

    let config = match preferred_config(&device) {
        Some(c) => c,
        None => device
            .default_input_config()
            .map_err(|e| format!("no default input config: {e}"))?,
    };

    let sample_rate = config.sample_rate();
    let channels = config.channels();
    let format = config.sample_format();

    let samples: Arc<StdMutex<Vec<f32>>> = Arc::new(StdMutex::new(Vec::new()));

    let stream = build_stream_dispatch(&device, config, format, samples.clone())?;
    stream.play().map_err(|e| format!("failed to start stream: {e}"))?;

    eprintln!(
        "[audio] input {sample_rate} Hz / {channels} ch / {format:?}"
    );

    let session = Session {
        stream,
        samples,
        sample_rate,
        channels,
    };
    *SESSION.lock().unwrap() = Some(session);
    Ok(())
}

pub fn stop_recording() -> Vec<f32> {
    let mut guard = SESSION.lock().unwrap();
    let Some(session) = guard.take() else {
        return Vec::new();
    };

    drop(session.stream);
    let raw = std::mem::take(&mut *session.samples.lock().unwrap());
    drop(guard);

    let mono = to_mono(&raw, session.channels);
    let at_16k = resample(&mono, session.sample_rate);
    trim_silence(&at_16k)
}

fn preferred_config(device: &Device) -> Option<SupportedStreamConfig> {
    let configs = device.supported_input_configs().ok()?;
    for range in configs {
        if range.channels() == 1 && range.sample_format() == SampleFormat::F32 {
            let (min, max) = (range.min_sample_rate(), range.max_sample_rate());
            if min <= TARGET_RATE && max >= TARGET_RATE {
                return Some(range.with_sample_rate(TARGET_RATE));
            }
        }
    }
    None
}

fn build_stream<T>(
    device: &Device,
    config: SupportedStreamConfig,
    format: SampleFormat,
    samples: Arc<StdMutex<Vec<f32>>>,
) -> Result<cpal::Stream, String>
where
    T: SizedSample + Sample,
    f32: FromSample<T>,
{
    let _ = format;
    let buffer = samples.clone();
    let err_tx = samples.clone();
    let stream = device.build_input_stream(
        config.config(),
        move |data: &[T], _: &cpal::InputCallbackInfo| {
            let mut buf = buffer.lock().unwrap();
            for s in data {
                buf.push((*s).to_sample::<f32>());
            }
        },
        move |err| {
            eprintln!("[audio] stream error: {err}");
            let _ = err_tx;
        },
        None,
    )
    .map_err(|e| e.to_string())?;
    Ok(stream)
}

macro_rules! dispatch {
    ($device:expr, $config:expr, $format:expr, $samples:expr) => {
        match $format {
            SampleFormat::F32 => build_stream::<f32>($device, $config, $format, $samples),
            SampleFormat::I16 => build_stream::<i16>($device, $config, $format, $samples),
            SampleFormat::U16 => build_stream::<u16>($device, $config, $format, $samples),
            SampleFormat::I8 => build_stream::<i8>($device, $config, $format, $samples),
            SampleFormat::U8 => build_stream::<u8>($device, $config, $format, $samples),
            SampleFormat::I32 => build_stream::<i32>($device, $config, $format, $samples),
            SampleFormat::U32 => build_stream::<u32>($device, $config, $format, $samples),
            SampleFormat::I64 => build_stream::<i64>($device, $config, $format, $samples),
            SampleFormat::U64 => build_stream::<u64>($device, $config, $format, $samples),
            SampleFormat::F64 => build_stream::<f64>($device, $config, $format, $samples),
            other => {
                eprintln!("[audio] unsupported sample format: {other:?}");
                Err("unsupported sample format".to_string())
            }
        }
    };
}

fn build_stream_dispatch(
    device: &Device,
    config: SupportedStreamConfig,
    format: SampleFormat,
    samples: Arc<StdMutex<Vec<f32>>>,
) -> Result<cpal::Stream, String> {
    dispatch!(device, config, format, samples)
}

fn to_mono(samples: &[f32], channels: u16) -> Vec<f32> {
    if channels <= 1 {
        return samples.to_vec();
    }
    let ch = channels as usize;
    let frames = samples.len() / ch;
    let mut out = Vec::with_capacity(frames);
    for frame in samples.chunks_exact(ch) {
        out.push(frame.iter().sum::<f32>() / ch as f32);
    }
    out
}

fn resample(input: &[f32], input_rate: u32) -> Vec<f32> {
    if input_rate == TARGET_RATE || input.is_empty() {
        return input.to_vec();
    }
    let ratio = input_rate as f64 / TARGET_RATE as f64;
    let cutoff = if ratio >= 1.0 { 0.5 / ratio } else { 0.5 } * 0.95;
    const TAPS: f64 = 32.0;

    let out_len = (input.len() as f64 / ratio).floor() as usize;
    let mut out = Vec::with_capacity(out_len);
    for i in 0..out_len {
        let center = i as f64 * ratio;
        let start = ((center - TAPS).ceil() as isize).max(0);
        let end = ((center + TAPS).floor() as isize).min(input.len() as isize - 1);

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
            sum += input[j as usize] as f64 * w;
            weight_sum += w;
            j += 1;
        }
        out.push((sum / weight_sum.max(1e-9)) as f32);
    }
    out
}

fn trim_silence(samples: &[f32]) -> Vec<f32> {
    let n = samples.len();
    let win = (TARGET_RATE as usize / 20).min(n);
    if win == 0 || n < win * 2 {
        return samples.to_vec();
    }
    const THRESHOLD: f32 = 0.005;
    let is_loud = |window: &[f32]| window.iter().any(|s| s.abs() > THRESHOLD);
    let step = (win / 4).max(1);

    let mut start = None;
    for i in (0..=n - win).step_by(step) {
        if is_loud(&samples[i..i + win]) {
            start = Some(i);
            break;
        }
    }

    let mut end = None;
    for i in (0..=n - win).rev() {
        if is_loud(&samples[i..i + win]) {
            end = Some(i + win);
            break;
        }
    }

    let (Some(start), Some(end)) = (start, end) else {
        return Vec::new();
    };
    if end <= start {
        return Vec::new();
    }

    let pad = (TARGET_RATE as usize / 10).min((end - start) / 2);
    let s = start.saturating_sub(pad);
    let e = (end + pad).min(n);
    samples[s..e].to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn read_s16_wav(path: &str) -> (Vec<f32>, u32, u16) {
        let bytes = std::fs::read(path).unwrap();
        let channels = u16::from_le_bytes([bytes[22], bytes[23]]) as usize;
        let rate = u32::from_le_bytes([bytes[24], bytes[25], bytes[26], bytes[27]]);

        let mut data = 12;
        while data + 8 <= bytes.len() {
            let size = u32::from_le_bytes([bytes[data + 4], bytes[data + 5], bytes[data + 6], bytes[data + 7]]) as usize;
            if &bytes[data..data + 4] == b"data" {
                data += 8;
                break;
            }
            data += 8 + size + (size % 2);
        }

        let mut samples = Vec::with_capacity((bytes.len() - data) / 2);
        for frame in bytes[data..].chunks_exact(2) {
            samples.push(i16::from_le_bytes([frame[0], frame[1]]) as f32 / i16::MAX as f32);
        }
        (samples, rate, channels as u16)
    }

    fn transcribe(samples: &[f32]) -> String {
        let model = std::env::var("V2T_MODEL")
            .unwrap_or_else(|_| "/home/harry/copilot/models/ggml-small.en.bin".into());
        crate::stt::Transcriber::new(std::path::Path::new(&model))
            .unwrap()
            .transcribe(samples)
            .unwrap()
    }

    #[test]
    fn resample_48k_recording_transcribes() {
        let base = env!("CARGO_MANIFEST_DIR");
        for (wav, expected) in [("what-is.wav", "time"), ("hello.wav", "hello")] {
            let path = format!("{base}/{wav}");
            if !std::path::Path::new(&path).exists() {
                eprintln!("skipping: {wav} missing");
                continue;
            }
            let (raw, rate, channels) = read_s16_wav(&path);
            let mono = to_mono(&raw, channels);
            let at_16k = resample(&mono, rate);
            let text = transcribe(&trim_silence(&at_16k)).to_lowercase();
            eprintln!("[audio test] {wav} -> {text:?}");
            assert!(text.contains(expected), "{wav} -> unexpected: {text:?}");
        }
    }

    #[test]
    fn resample_sine_matches_frequency() {
        let rate = 48_000;
        let freq = 440.0;
        let input: Vec<f32> = (0..rate * 2)
            .map(|i| (2.0 * std::f64::consts::PI * freq * i as f64 / rate as f64).sin() as f32)
            .collect();
        let out = resample(&input, rate);
        assert_eq!(out.len(), input.len() / 3);

        let crossings = out.windows(2).filter(|w| w[0] <= 0.0 && w[1] > 0.0).count();
        let measured = crossings as f64 * TARGET_RATE as f64 / out.len() as f64;
        eprintln!("[audio test] measured {measured:.1} Hz (expected ~{freq})");
        assert!((measured - freq).abs() < 5.0, "bad resample frequency: {measured}");
    }

    #[test]
    fn to_mono_averages_channels() {
        let stereo = vec![1.0, 3.0, 5.0, 7.0];
        let mono = to_mono(&stereo, 2);
        assert_eq!(mono, vec![2.0, 6.0]);

        let passthrough = to_mono(&stereo, 1);
        assert_eq!(passthrough, stereo);
    }

    #[test]
    fn resample_passthrough_at_target_rate() {
        let input = vec![0.1, 0.2, 0.3];
        assert_eq!(resample(&input, TARGET_RATE), input);
        assert!(resample(&[], 48_000).is_empty());
    }

    #[test]
    fn trim_silence_keeps_loud_middle() {
        let mut samples = vec![0.0; 4000];
        for i in 2000..2400 {
            samples[i] = 0.5;
        }
        let trimmed = trim_silence(&samples);
        assert!(!trimmed.is_empty());
        assert!(trimmed.len() < samples.len());
        assert!(trimmed.iter().any(|s| s.abs() > 0.005), "speech was trimmed away");

        let all_silent = vec![0.0; 4000];
        assert!(trim_silence(&all_silent).is_empty());
    }
}
