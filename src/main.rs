use alsa::{
    pcm::{Format, HwParams, PCM},
    ValueOr,
};
use anyhow::Result;
use std::f64::consts::PI;

const DEFAULT_DURATION: usize = 10;
const DEFAULT_SAMPLE_RATE: usize = 48000;
const MAX_FREQ: f64 = 24_000.0;

const WAVE_SPEC: hound::WavSpec = hound::WavSpec {
    channels: 1,
    sample_rate: DEFAULT_SAMPLE_RATE as u32,
    bits_per_sample: 32,
    sample_format: hound::SampleFormat::Float,
};

fn main() -> Result<()> {
    let pcm = PCM::new("default", alsa::Direction::Playback, false)?;

    // Set hardware parameters:
    let hwp = HwParams::any(&pcm)?;
    hwp.set_channels(1)?;
    hwp.set_rate(DEFAULT_SAMPLE_RATE as u32, ValueOr::Nearest)?;
    hwp.set_format(Format::float())?;
    pcm.hw_params(&hwp)?;
    let io = pcm.io_f32()?;

    // Make a sine wave
    let total_samples = DEFAULT_SAMPLE_RATE * DEFAULT_DURATION;
    let freq_slope = MAX_FREQ / total_samples as f64;
    let samples = (0..total_samples).into_iter().map(|i| i as f64).map(|i| {
        let freq = i * freq_slope;
        let time = i / total_samples as f64;
        (2.0 * PI * freq * time).sin()
    });

    let mut play_samples = samples.clone();
    for _ in 0..DEFAULT_DURATION {
        let buf = play_samples
            .by_ref()
            .map(|v| v as f32)
            .take(DEFAULT_SAMPLE_RATE)
            .collect::<Vec<_>>();
        assert_eq!(buf.len(), DEFAULT_SAMPLE_RATE);
        assert_eq!(io.writei(&buf)?, DEFAULT_SAMPLE_RATE);
    }

    pcm.drain()?;

    let mut writer = hound::WavWriter::create("sine.wav", WAVE_SPEC)?;
    for s in samples {
        writer.write_sample(s as f32)?;
    }

    Ok(())
}
