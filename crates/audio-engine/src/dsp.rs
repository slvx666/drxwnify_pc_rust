pub struct DspProcessor {
    pub volume: f32,
    pub balance: f32,
    pub fade: f32,
    pub normalize_enabled: bool,
    pub skip_silence_enabled: bool,
    smoothed_rms: f32,
    norm_gain: f32,
    silence_tail_frames: usize,
}

impl DspProcessor {
    pub fn new() -> Self {
        Self {
            volume: 0.8,
            balance: 0.0,
            fade: 1.0,
            normalize_enabled: true,
            skip_silence_enabled: false,
            smoothed_rms: 0.08,
            norm_gain: 1.0,
            silence_tail_frames: 0,
        }
    }

    pub fn process_stereo_frame(&mut self, left: i16, right: i16) -> (i16, i16) {
        let eff_norm_gain = if self.normalize_enabled { self.norm_gain } else { 1.0 };
        let base_gain = (self.volume.clamp(0.0, 1.0) as f64).powf(1.7) as f32 * self.fade * eff_norm_gain;

        let gain_l = base_gain * if self.balance > 0.0 { 1.0 - self.balance } else { 1.0 };
        let gain_r = base_gain * if self.balance < 0.0 { 1.0 + self.balance } else { 1.0 };

        let out_l = (left as f32 * gain_l).clamp(i16::MIN as f32, i16::MAX as f32) as i16;
        let out_r = (right as f32 * gain_r).clamp(i16::MIN as f32, i16::MAX as f32) as i16;

        (out_l, out_r)
    }

    pub fn update_rms_from_frame(&mut self, samples: &[(i16, i16)]) {
        if samples.is_empty() || !self.normalize_enabled {
            return;
        }

        let mut sum_sq: f64 = 0.0;
        for &(l, r) in samples {
            sum_sq += (l as f64) * (l as f64) + (r as f64) * (r as f64);
        }

        let frame_rms = ((sum_sq / (2.0 * samples.len() as f64)).sqrt() / 32768.0) as f32;

        if frame_rms > 0.005 {
            self.smoothed_rms = self.smoothed_rms * 0.96 + frame_rms * 0.04;
            let target_gain = (0.16 / self.smoothed_rms.max(0.01)).clamp(0.5, 2.2);
            self.norm_gain = self.norm_gain * 0.94 + target_gain * 0.06;
        }
    }

    pub fn check_lead_silence(&self, samples: &[(i16, i16)], position_ms: i64) -> bool {
        if !self.skip_silence_enabled || position_ms > 20_000 {
            return false;
        }

        let max_peak = samples
            .iter()
            .map(|&(l, r)| l.abs().max(r.abs()))
            .max()
            .unwrap_or(0);

        max_peak < 300
    }

    pub fn check_tail_silence(&mut self, samples: &[(i16, i16)], position_ms: i64, duration_ms: i64) -> bool {
        if !self.skip_silence_enabled || duration_ms <= 0 || position_ms < duration_ms - 10_000 {
            self.silence_tail_frames = 0;
            return false;
        }

        let max_peak = samples
            .iter()
            .map(|&(l, r)| l.abs().max(r.abs()))
            .max()
            .unwrap_or(0);

        if max_peak < 250 {
            self.silence_tail_frames += 1;
            self.silence_tail_frames >= 35
        } else {
            self.silence_tail_frames = 0;
            false
        }
    }
}

impl Default for DspProcessor {
    fn default() -> Self {
        Self::new()
    }
}
