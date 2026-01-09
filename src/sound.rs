// SPDX-License-Identifier: MPL-2.0

use std::io::Cursor;

/// Generates a simple sine wave WAV file in memory
fn generate_sine_wave(frequency: f32, duration_secs: f32) -> Vec<u8> {
    const SAMPLE_RATE: u32 = 44100;
    const CHANNELS: u16 = 1;
    const BITS_PER_SAMPLE: u16 = 16;

    let num_samples = (SAMPLE_RATE as f32 * duration_secs) as u32;
    let mut samples = Vec::with_capacity(num_samples as usize);

    // Generate sine wave samples
    for i in 0..num_samples {
        let t = i as f32 / SAMPLE_RATE as f32;
        let sample = (2.0 * std::f32::consts::PI * frequency * t).sin();
        let sample_i16 = (sample * 32767.0) as i16;
        samples.push(sample_i16);
    }

    // Build WAV file
    let data_size = samples.len() * 2; // 2 bytes per sample
    let file_size = 36 + data_size;

    let mut wav = Vec::with_capacity(44 + data_size);

    // RIFF header
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&(file_size as u32).to_le_bytes());
    wav.extend_from_slice(b"WAVE");

    // fmt chunk
    wav.extend_from_slice(b"fmt ");
    wav.extend_from_slice(&16u32.to_le_bytes()); // chunk size
    wav.extend_from_slice(&1u16.to_le_bytes()); // audio format (PCM)
    wav.extend_from_slice(&CHANNELS.to_le_bytes());
    wav.extend_from_slice(&SAMPLE_RATE.to_le_bytes());
    wav.extend_from_slice(
        &(SAMPLE_RATE * CHANNELS as u32 * BITS_PER_SAMPLE as u32 / 8).to_le_bytes(),
    ); // byte rate
    wav.extend_from_slice(&(CHANNELS * BITS_PER_SAMPLE / 8).to_le_bytes()); // block align
    wav.extend_from_slice(&BITS_PER_SAMPLE.to_le_bytes());

    // data chunk
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&(data_size as u32).to_le_bytes());
    for sample in samples {
        wav.extend_from_slice(&sample.to_le_bytes());
    }

    wav
}

/// Plays a sound from WAV data
pub fn play_sound(wav_data: &[u8]) {
    // Silent fail if audio system unavailable
    let _ = std::thread::spawn({
        let wav_data = wav_data.to_vec();
        move || {
            if let Ok((_stream, handle)) = rodio::OutputStream::try_default() {
                if let Ok(sink) = rodio::Sink::try_new(&handle) {
                    if let Ok(source) = rodio::Decoder::new(Cursor::new(wav_data)) {
                        sink.append(source);
                        sink.sleep_until_end();
                        // Give audio system extra time to flush buffers before dropping stream
                        // Longest sound is 0.5s, so 600ms ensures full playback
                        std::thread::sleep(std::time::Duration::from_millis(600));
                    }
                }
            }
        }
    });
}

pub fn play_work_sound() {
    let wav_data = work_sound();
    play_sound(&wav_data);
}

pub fn play_break_sound() {
    let wav_data = break_sound();
    play_sound(&wav_data);
}

/// Work sound: 800Hz sine wave, 0.3 seconds
fn work_sound() -> Vec<u8> {
    generate_sine_wave(800.0, 0.3)
}

/// Break sound: 600Hz sine wave, 0.5 seconds
fn break_sound() -> Vec<u8> {
    generate_sine_wave(600.0, 0.5)
}
