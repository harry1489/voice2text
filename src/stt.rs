use std::path::Path;

use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

pub struct Transcriber {
    ctx: WhisperContext,
}

impl Transcriber {
    pub fn new(model_path: &Path) -> Result<Self, String> {
        if !model_path.exists() {
            return Err(format!(
                "whisper model not found at {} - run ./install.sh to download it",
                model_path.display()
            ));
        }

        let mut cparams = WhisperContextParameters::default();
        cparams.use_gpu = false;
        let ctx = WhisperContext::new_with_params(model_path, cparams)
            .map_err(|e| format!("failed to load model: {e}"))?;

        Ok(Self { ctx })
    }

    pub fn transcribe(&self, samples: &[f32]) -> Result<String, String> {
        let mut state = self.ctx.create_state().map_err(|e| e.to_string())?;

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

        state.full(params, samples).map_err(|e| e.to_string())?;

        let mut out = String::new();
        for seg in state.as_iter() {
            if seg.no_speech_probability() < 0.6 {
                if let Ok(text) = seg.to_str() {
                    out.push_str(text);
                }
            }
        }
        Ok(out.trim().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn transcribes_known_sample() {
        let model = std::env::var("V2T_MODEL")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("/home/harry/copilot/models/ggml-small.en.bin"));
        let wav = std::env::var("V2T_SAMPLE")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("/home/harry/copilot/samples/jfk.wav"));

        if !model.exists() || !wav.exists() {
            eprintln!("skipping: model or sample missing");
            return;
        }

        let transcriber = Transcriber::new(&model).unwrap();
        let samples = read_wav_mono_16k(&wav);
        let text = transcriber.transcribe(&samples).unwrap();

        eprintln!("[test] transcribed: {text:?}");
        assert!(
            text.to_lowercase().contains("fellow") || text.to_lowercase().contains("american"),
            "unexpected transcription: {text:?}"
        );
    }

    fn read_wav_mono_16k(path: &Path) -> Vec<f32> {
        let bytes = std::fs::read(path).unwrap();
        assert_eq!(&bytes[..4], b"RIFF", "not a RIFF wav");
        let channels = u16::from_le_bytes([bytes[22], bytes[23]]) as usize;
        let rate = u32::from_le_bytes([bytes[24], bytes[25], bytes[26], bytes[27]]);
        let bits = u16::from_le_bytes([bytes[34], bytes[35]]);
        assert_eq!(rate, 16_000, "expected 16 kHz sample");
        assert_eq!(bits, 16, "expected 16-bit pcm");

        let data = &bytes[44..];
        let mut out = Vec::with_capacity(data.len() / 2);
        for frame in data.chunks_exact(channels * 2) {
            let sample =
                i16::from_le_bytes([frame[0], frame[1]]) as f32 / i16::MAX as f32;
            out.push(sample);
        }
        out
    }
}
