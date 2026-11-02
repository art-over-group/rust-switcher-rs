//! Sound feedback playback via Win32 `PlaySoundW`.
//!
//! The cues are synthesized once into in-memory WAV buffers
//! (`rust_switcher_core::sound`) and played asynchronously on demand. The
//! buffers are kept in statics for the process lifetime because `SND_ASYNC`
//! requires the memory to stay valid until playback completes.

use std::sync::OnceLock;

use rust_switcher_core::sound;

use windows::Win32::Media::Audio::{PlaySoundW, SND_ASYNC, SND_FLAGS, SND_MEMORY, SND_NODEFAULT};
use windows::core::PCWSTR;

/// Which cue to play.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum SoundCue {
    /// Short quiet tick after a successful conversion.
    ConversionTick,
}

fn cue_buffer(cue: SoundCue) -> &'static Vec<u8> {
    static TICK: OnceLock<Vec<u8>> = OnceLock::new();

    match cue {
        SoundCue::ConversionTick => TICK.get_or_init(sound::conversion_tick_wav),
    }
}

/// Plays a cue asynchronously. Never blocks the UI thread; playback failures
/// are silent by design — sound is best-effort feedback.
pub fn play(cue: SoundCue) {
    let buffer = cue_buffer(cue);
    let ptr = PCWSTR(buffer.as_ptr().cast());
    let flags = SND_FLAGS(SND_MEMORY.0 | SND_ASYNC.0 | SND_NODEFAULT.0);
    unsafe {
        let _ = PlaySoundW(ptr, None, flags);
    }
}
