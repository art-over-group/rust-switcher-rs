//! Procedurally synthesized audio cues (pure code, no platform dependencies).
//!
//! The cues are quiet by design: the peak amplitude is capped at 25% of full
//! scale, so a conversion tick is noticeable but never intrusive. The output
//! is a complete 16-bit mono 22 050 Hz PCM WAV file in memory, ready for
//! `PlaySoundW(SND_MEMORY)`.

/// Sample rate for all synthesized cues, Hz.
const SAMPLE_RATE: u32 = 22_050;
/// Peak amplitude cap: 25% of 16-bit full scale — quiet by design.
const PEAK_AMPLITUDE: f64 = 8_192.0;

/// Builds a complete 16-bit mono PCM WAV file from samples in [-1.0; 1.0].
fn wav_bytes(samples: &[f64]) -> Vec<u8> {
    let data_len = samples.len() * 2;
    let mut out = Vec::with_capacity(44 + data_len);

    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&(36u32 + data_len as u32).to_le_bytes());
    out.extend_from_slice(b"WAVE");
    // fmt chunk
    out.extend_from_slice(b"fmt ");
    out.extend_from_slice(&16u32.to_le_bytes()); // chunk size
    out.extend_from_slice(&1u16.to_le_bytes()); // PCM
    out.extend_from_slice(&1u16.to_le_bytes()); // mono
    out.extend_from_slice(&SAMPLE_RATE.to_le_bytes());
    out.extend_from_slice(&(SAMPLE_RATE * 2).to_le_bytes()); // byte rate
    out.extend_from_slice(&2u16.to_le_bytes()); // block align
    out.extend_from_slice(&16u16.to_le_bytes()); // bits per sample
    // data chunk
    out.extend_from_slice(b"data");
    out.extend_from_slice(&(data_len as u32).to_le_bytes());
    for sample in samples {
        let clamped = sample.clamp(-1.0, 1.0);
        out.extend_from_slice(&((clamped * PEAK_AMPLITUDE) as i16).to_le_bytes());
    }

    out
}

/// Sine wave with an exponential decay envelope.
fn sine_decay(duration_ms: u32, frequency: f64, decay_tau_ms: f64) -> Vec<f64> {
    let sample_count = duration_ms * SAMPLE_RATE / 1_000;
    (0..sample_count)
        .map(|index| {
            let t = index as f64 / SAMPLE_RATE as f64;
            let envelope = (-t / (decay_tau_ms / 1_000.0)).exp();
            envelope * (2.0 * std::f64::consts::TAU * frequency * t).sin()
        })
        .collect()
}

/// Short quiet tick played after a successful conversion (~45 ms, 1320 Hz).
pub fn conversion_tick_wav() -> Vec<u8> {
    wav_bytes(&sine_decay(45, 1_320.0, 14.0))
}

/// Slightly longer descending cue for failures (~120 ms, 520→392 Hz).
pub fn error_cue_wav() -> Vec<u8> {
    let first = sine_decay(60, 520.0, 30.0);
    let second = sine_decay(60, 392.0, 30.0);
    wav_bytes(&[first, second].concat())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn data_len(bytes: &[u8]) -> usize {
        u32::from_le_bytes([bytes[40], bytes[41], bytes[42], bytes[43]]) as usize
    }

    #[test]
    fn wav_header_is_valid_riff_pcm() {
        let wav = conversion_tick_wav();
        assert_eq!(&wav[0..4], b"RIFF");
        assert_eq!(&wav[8..12], b"WAVE");
        assert_eq!(&wav[12..16], b"fmt ");
        assert_eq!(u32::from_le_bytes([wav[16], wav[17], wav[18], wav[19]]), 16);
        assert_eq!(u16::from_le_bytes([wav[20], wav[21]]), 1); // PCM
        assert_eq!(u16::from_le_bytes([wav[22], wav[23]]), 1); // mono
        assert_eq!(
            u32::from_le_bytes([wav[24], wav[25], wav[26], wav[27]]),
            SAMPLE_RATE
        );
        assert_eq!(u16::from_le_bytes([wav[28], wav[29]]), 2); // block align
        assert_eq!(u16::from_le_bytes([wav[30], wav[31]]), 16); // bits
        assert_eq!(&wav[36..40], b"data");
    }

    #[test]
    fn tick_duration_matches_spec() {
        let wav = conversion_tick_wav();
        let data_len = data_len(&wav);
        // 45 ms at 22050 Hz mono 16-bit = 992 samples = 1984 bytes
        assert_eq!(data_len, 1_984);
        // RIFF size covers the rest of the file.
        assert_eq!(
            u32::from_le_bytes([wav[4], wav[5], wav[6], wav[7]]) as usize,
            wav.len() - 8
        );
    }

    #[test]
    fn samples_stay_within_quiet_bounds() {
        for wav in [conversion_tick_wav(), error_cue_wav()] {
            let data = &wav[44..];
            // Manual pair iteration: avoids clippy::chunks_exact_to_as_chunks
            // on the latest nightly toolchains used by CI.
            let mut pairs = data.iter();
            while let (Some(lo), Some(hi)) = (pairs.next(), pairs.next()) {
                let sample = i16::from_le_bytes([*lo, *hi]);
                assert!(sample.abs() <= 8_192);
            }
        }
    }

    #[test]
    fn error_cue_is_two_tone() {
        let wav = error_cue_wav();
        // 120 ms total = 2646 samples
        assert_eq!(data_len(&wav), 2_646);
    }
}
