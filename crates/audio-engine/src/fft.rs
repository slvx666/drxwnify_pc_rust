use rustfft::num_complex::Complex;
use rustfft::FftPlanner;
use std::sync::Arc;

pub const FFT_SIZE: usize = 1024;
pub const NUM_BANDS: usize = 32;

pub struct SpectrumAnalyzer {
    fft: Arc<dyn rustfft::Fft<f32>>,
    hann_window: [f32; FFT_SIZE],
    ring_buffer: Vec<f32>,
    band_indices: [(usize, usize); NUM_BANDS],
    smooth_bands: [f32; NUM_BANDS],
}

impl SpectrumAnalyzer {
    pub fn new() -> Self {
        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(FFT_SIZE);

        let mut hann_window = [0.0; FFT_SIZE];
        for i in 0..FFT_SIZE {
            hann_window[i] = 0.5 - 0.5 * ((2.0 * std::f32::consts::PI * i as f32) / (FFT_SIZE as f32 - 1.0)).cos();
        }

        // Calculate 32 logarithmic bands between 40 Hz and 20,000 Hz (sample rate 48000 Hz)
        // Bin resolution = 48000 / 1024 = 46.875 Hz per bin
        let min_freq: f32 = 40.0;
        let max_freq: f32 = 20_000.0;
        let bin_res = 48_000.0 / FFT_SIZE as f32;

        let mut band_indices = [(0, 0); NUM_BANDS];
        for b in 0..NUM_BANDS {
            let f_low = min_freq * (max_freq / min_freq).powf(b as f32 / NUM_BANDS as f32);
            let f_high = min_freq * (max_freq / min_freq).powf((b + 1) as f32 / NUM_BANDS as f32);

            let mut start_bin = (f_low / bin_res).floor() as usize;
            let mut end_bin = (f_high / bin_res).ceil() as usize;

            if start_bin >= FFT_SIZE / 2 {
                start_bin = (FFT_SIZE / 2) - 1;
            }
            if end_bin >= FFT_SIZE / 2 {
                end_bin = FFT_SIZE / 2;
            }
            if end_bin <= start_bin {
                end_bin = start_bin + 1;
            }

            band_indices[b] = (start_bin, end_bin);
        }

        Self {
            fft,
            hann_window,
            ring_buffer: Vec::with_capacity(FFT_SIZE * 2),
            band_indices,
            smooth_bands: [0.0; NUM_BANDS],
        }
    }

    pub fn push_samples(&mut self, samples: &[(i16, i16)]) {
        for &(l, r) in samples {
            // Mono mix normalized to [-1.0, 1.0]
            let mono = ((l as f32 + r as f32) * 0.5) / 32768.0;
            self.ring_buffer.push(mono);
        }

        if self.ring_buffer.len() > FFT_SIZE * 4 {
            let overflow = self.ring_buffer.len() - FFT_SIZE;
            self.ring_buffer.drain(0..overflow);
        }
    }

    pub fn compute_spectrum(&mut self) -> [f32; NUM_BANDS] {
        if self.ring_buffer.len() < FFT_SIZE {
            return self.smooth_bands;
        }

        let start_idx = self.ring_buffer.len() - FFT_SIZE;
        let slice = &self.ring_buffer[start_idx..];

        let mut input: Vec<Complex<f32>> = slice
            .iter()
            .enumerate()
            .map(|(i, &val)| Complex {
                re: val * self.hann_window[i],
                im: 0.0,
            })
            .collect();

        self.fft.process(&mut input);

        for (b, &(start_bin, end_bin)) in self.band_indices.iter().enumerate() {
            let mut acc: f32 = 0.0;
            let count = (end_bin - start_bin).max(1);

            for i in start_bin..end_bin {
                let mag = input[i].norm() / (FFT_SIZE as f32);
                acc += mag;
            }
            acc /= count as f32;

            // dB level normalized from [-60 dB, 0 dB] -> [0.0, 1.0]
            let db = 20.0 * (acc + 1e-9).log10();
            let v = ((db + 60.0) / 60.0).clamp(0.0, 1.0);

            // Smooth decay: smooth * 0.82 + v * 0.18
            self.smooth_bands[b] = v.max(self.smooth_bands[b] * 0.82 + v * 0.18);
        }

        self.smooth_bands
    }
}

impl Default for SpectrumAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}
