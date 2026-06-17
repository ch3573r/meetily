// nemotron_engine/features.rs
//
// Pure-Rust log-mel feature extraction for the Nemotron 3.5 ASR streaming model.
//
// Nemotron ships no preprocessor ONNX (unlike Parakeet's nemo128.onnx), so we
// compute the 128-bin log-mel spectrogram here, matching the model's
// `audio_processor_config.json` exactly:
//
//   sample_rate 16000  n_fft 512  hop_length 160  win_length 400  window hann
//   n_mels 128  fmin 0  fmax 8000  preemphasis 0.97  mag_power 2.0
//   center true  log = ln(x + 1e-10)  normalize "NA" (none)
//
// This mirrors NeMo's FilterbankFeatures / librosa mel (htk=false, slaney norm)
// and torch.stft(center=true) reflect padding. Exact numerical parity against a
// NeMo reference vector is still pending (see NEMOTRON_IMPLEMENTATION_PLAN.md
// §6) but the shape/scale are correct and unit-tested.

use realfft::RealFftPlanner;

pub const SAMPLE_RATE: usize = 16000;
pub const N_FFT: usize = 512;
pub const HOP_LENGTH: usize = 160;
pub const WIN_LENGTH: usize = 400;
pub const N_MELS: usize = 128;
pub const FMIN: f32 = 0.0;
pub const FMAX: f32 = 8000.0;
pub const PREEMPH: f32 = 0.97;
pub const LOG_GUARD: f32 = 1e-10;

/// Precomputed, reusable feature extractor (FFT plan + window + mel filterbank).
pub struct MelExtractor {
    planner: RealFftPlanner<f32>,
    /// Hann window of length WIN_LENGTH, zero-padded/centered into N_FFT.
    window: Vec<f32>,
    /// Mel filterbank: N_MELS rows, each (N_FFT/2 + 1) weights.
    mel_filters: Vec<Vec<f32>>,
}

impl MelExtractor {
    pub fn new() -> Self {
        Self {
            planner: RealFftPlanner::<f32>::new(),
            window: build_centered_hann(WIN_LENGTH, N_FFT),
            mel_filters: build_mel_filterbank(),
        }
    }

    /// Compute log-mel features for a mono 16 kHz waveform.
    ///
    /// Returns features as `[N_MELS][n_frames]` (mel-major), ready to reshape
    /// into the encoder's `[1, 128, T]` `audio_signal` input.
    pub fn compute(&mut self, samples: &[f32]) -> Vec<Vec<f32>> {
        if samples.is_empty() {
            return vec![Vec::new(); N_MELS];
        }

        // 1. Pre-emphasis: y[t] = x[t] - 0.97 * x[t-1].
        let mut sig = vec![0.0f32; samples.len()];
        sig[0] = samples[0];
        for t in 1..samples.len() {
            sig[t] = samples[t] - PREEMPH * samples[t - 1];
        }

        // 2. Center padding (torch.stft center=true → reflect pad by N_FFT/2).
        let pad = N_FFT / 2;
        let padded = reflect_pad(&sig, pad);

        // 3. Frame, window, FFT, power, mel, log.
        let n_bins = N_FFT / 2 + 1;
        let n_frames = if padded.len() >= N_FFT {
            1 + (padded.len() - N_FFT) / HOP_LENGTH
        } else {
            0
        };

        let r2c = self.planner.plan_fft_forward(N_FFT);
        let mut frame_buf = r2c.make_input_vec();
        let mut spectrum = r2c.make_output_vec();

        // Output is mel-major so the caller can build [128, T] directly.
        let mut out: Vec<Vec<f32>> = vec![Vec::with_capacity(n_frames); N_MELS];

        for f in 0..n_frames {
            let start = f * HOP_LENGTH;
            for i in 0..N_FFT {
                frame_buf[i] = padded[start + i] * self.window[i];
            }
            r2c.process(&mut frame_buf, &mut spectrum)
                .expect("rfft process");

            // Power spectrum (mag_power = 2.0).
            let mut power = vec![0.0f32; n_bins];
            for (b, c) in spectrum.iter().enumerate() {
                power[b] = c.norm_sqr();
            }

            // Mel projection + log guard.
            for m in 0..N_MELS {
                let filt = &self.mel_filters[m];
                let mut acc = 0.0f32;
                for b in 0..n_bins {
                    acc += filt[b] * power[b];
                }
                out[m].push((acc + LOG_GUARD).ln());
            }
        }

        out
    }
}

impl Default for MelExtractor {
    fn default() -> Self {
        Self::new()
    }
}

/// Hann window of `win` samples, centered inside an `n_fft` frame (zeros on the
/// outer `(n_fft - win)/2` samples each side) — matches torch.stft window
/// padding when `win_length < n_fft`.
fn build_centered_hann(win: usize, n_fft: usize) -> Vec<f32> {
    let mut w = vec![0.0f32; n_fft];
    let off = (n_fft - win) / 2;
    for i in 0..win {
        // Periodic Hann (matches torch/librosa default).
        let x = (std::f32::consts::PI * i as f32 / win as f32).sin();
        w[off + i] = x * x;
    }
    w
}

/// Reflect-pad a signal by `pad` samples on each end (mirror without repeating
/// the edge sample), matching torch.stft(center=true).
fn reflect_pad(sig: &[f32], pad: usize) -> Vec<f32> {
    let n = sig.len();
    let mut out = Vec::with_capacity(n + 2 * pad);
    for i in 0..pad {
        // reflect: sig[pad - i] ... index mirrors around sig[0]
        let idx = (pad - i).min(n.saturating_sub(1));
        out.push(sig[idx]);
    }
    out.extend_from_slice(sig);
    for i in 0..pad {
        let idx = n.saturating_sub(2).saturating_sub(i);
        out.push(sig[idx.min(n - 1)]);
    }
    out
}

fn hz_to_mel_slaney(hz: f32) -> f32 {
    let f_sp = 200.0 / 3.0;
    let min_log_hz = 1000.0;
    let min_log_mel = min_log_hz / f_sp; // 15.0
    let logstep = (6.4f32).ln() / 27.0;
    if hz < min_log_hz {
        hz / f_sp
    } else {
        min_log_mel + (hz / min_log_hz).ln() / logstep
    }
}

fn mel_to_hz_slaney(mel: f32) -> f32 {
    let f_sp = 200.0 / 3.0;
    let min_log_hz = 1000.0;
    let min_log_mel = min_log_hz / f_sp; // 15.0
    let logstep = (6.4f32).ln() / 27.0;
    if mel < min_log_mel {
        f_sp * mel
    } else {
        min_log_hz * (logstep * (mel - min_log_mel)).exp()
    }
}

/// librosa-style mel filterbank (htk=false, slaney norm), N_MELS x (N_FFT/2+1).
fn build_mel_filterbank() -> Vec<Vec<f32>> {
    let n_bins = N_FFT / 2 + 1;

    // FFT bin center frequencies.
    let fft_freqs: Vec<f32> = (0..n_bins)
        .map(|b| b as f32 * SAMPLE_RATE as f32 / N_FFT as f32)
        .collect();

    // N_MELS + 2 mel points → hz points.
    let mel_min = hz_to_mel_slaney(FMIN);
    let mel_max = hz_to_mel_slaney(FMAX);
    let mel_points: Vec<f32> = (0..N_MELS + 2)
        .map(|i| mel_min + (mel_max - mel_min) * i as f32 / (N_MELS + 1) as f32)
        .collect();
    let hz_points: Vec<f32> = mel_points.iter().map(|&m| mel_to_hz_slaney(m)).collect();

    let mut filters = vec![vec![0.0f32; n_bins]; N_MELS];
    for m in 0..N_MELS {
        let lower = hz_points[m];
        let center = hz_points[m + 1];
        let upper = hz_points[m + 2];
        // Slaney normalization: 2 / (upper - lower).
        let enorm = 2.0 / (upper - lower);
        for (b, &freq) in fft_freqs.iter().enumerate() {
            let w = if freq < lower || freq > upper {
                0.0
            } else if freq <= center {
                (freq - lower) / (center - lower)
            } else {
                (upper - freq) / (upper - center)
            };
            filters[m][b] = w.max(0.0) * enorm;
        }
    }
    filters
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mel_filterbank_shape_and_norm() {
        let fb = build_mel_filterbank();
        assert_eq!(fb.len(), N_MELS);
        assert_eq!(fb[0].len(), N_FFT / 2 + 1);
        // Every filter should have some positive weight and be finite.
        for filt in &fb {
            let sum: f32 = filt.iter().sum();
            assert!(sum > 0.0, "mel filter had no weight");
            assert!(filt.iter().all(|w| w.is_finite()));
        }
    }

    #[test]
    fn compute_produces_expected_frame_count() {
        // 1 second of 440 Hz tone at 16 kHz.
        let n = SAMPLE_RATE;
        let tone: Vec<f32> = (0..n)
            .map(|i| (2.0 * std::f32::consts::PI * 440.0 * i as f32 / SAMPLE_RATE as f32).sin())
            .collect();
        let mut ex = MelExtractor::new();
        let feats = ex.compute(&tone);
        assert_eq!(feats.len(), N_MELS);
        // center=true: n_frames ≈ 1 + n / hop.
        let frames = feats[0].len();
        let expected = 1 + n / HOP_LENGTH;
        assert!(
            (frames as i64 - expected as i64).abs() <= 1,
            "got {frames} frames, expected ~{expected}"
        );
        // All features finite.
        assert!(feats.iter().all(|row| row.iter().all(|v| v.is_finite())));
    }

    #[test]
    fn silence_is_near_log_floor() {
        let mut ex = MelExtractor::new();
        let feats = ex.compute(&vec![0.0f32; SAMPLE_RATE / 2]);
        // log(0 + 1e-10) ≈ -23.03; silence should sit near the floor.
        let max = feats
            .iter()
            .flat_map(|r| r.iter())
            .cloned()
            .fold(f32::MIN, f32::max);
        assert!(max < -10.0, "silence mel max too high: {max}");
    }
}
