pub mod dsp;
pub mod fft;
pub mod wasapi;
pub mod decoder;

pub use dsp::DspProcessor;
pub use fft::SpectrumAnalyzer;
pub use wasapi::WasapiOutput;
pub use decoder::{AudioSource, AudioDecoder};

use drxwnify_core::{CoreEvent, DrxwnifyResult, EventBus};
use drxwnify_shared_models::*;
use parking_lot::Mutex;
use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};
use std::sync::Arc;
use std::time::Instant;

pub struct AudioEngine {
    event_bus: EventBus,
    output: Option<WasapiOutput>,
    dsp: Arc<Mutex<DspProcessor>>,
    spectrum: Arc<Mutex<SpectrumAnalyzer>>,
    decoder: Arc<Mutex<Option<AudioDecoder>>>,
    is_playing: Arc<AtomicBool>,
    position_ms: Arc<AtomicI64>,
    duration_sec: Arc<AtomicI64>,
    last_sample_instant: Arc<Mutex<Instant>>,
}

impl AudioEngine {
    pub fn new(event_bus: EventBus, device_name: Option<&str>) -> Self {
        let output = match WasapiOutput::new(device_name) {
            Ok(out) => Some(out),
            Err(e) => {
                tracing::warn!("Could not initialize WASAPI output: {e}, falling back to silent engine");
                None
            }
        };

        Self {
            event_bus,
            output,
            dsp: Arc::new(Mutex::new(DspProcessor::new())),
            spectrum: Arc::new(Mutex::new(SpectrumAnalyzer::new())),
            decoder: Arc::new(Mutex::new(None)),
            is_playing: Arc::new(AtomicBool::new(false)),
            position_ms: Arc::new(AtomicI64::new(0)),
            duration_sec: Arc::new(AtomicI64::new(0)),
            last_sample_instant: Arc::new(Mutex::new(Instant::now())),
        }
    }

    pub fn play(&self) {
        self.is_playing.store(true, Ordering::SeqCst);
        self.event_bus.emit(CoreEvent::StateChanged(PlaybackState::Playing));
    }

    pub fn pause(&self) {
        self.is_playing.store(false, Ordering::SeqCst);
        self.event_bus.emit(CoreEvent::StateChanged(PlaybackState::Paused));
    }

    pub fn stop(&self) {
        self.is_playing.store(false, Ordering::SeqCst);
        self.position_ms.store(0, Ordering::SeqCst);
        if let Some(ref out) = self.output {
            out.clear();
        }
        self.event_bus.emit(CoreEvent::StateChanged(PlaybackState::Stopped));
    }

    pub fn seek(&self, position_ms: i64) -> DrxwnifyResult<()> {
        self.position_ms.store(position_ms, Ordering::SeqCst);
        if let Some(ref mut dec) = *self.decoder.lock() {
            dec.seek(position_ms)?;
        }
        if let Some(ref out) = self.output {
            out.clear();
        }
        Ok(())
    }

    pub fn set_volume(&self, volume: f32) {
        self.dsp.lock().volume = volume;
    }

    pub fn set_balance(&self, balance: f32) {
        self.dsp.lock().balance = balance;
    }

    pub fn set_normalize(&self, normalize: bool) {
        self.dsp.lock().normalize_enabled = normalize;
    }

    pub fn set_skip_silence(&self, skip: bool) {
        self.dsp.lock().skip_silence_enabled = skip;
    }

    pub fn attach_local(&self, path: std::path::PathBuf) {
        if let Some(ref mut dec) = *self.decoder.lock() {
            dec.attach_local_file(path);
        }
    }

    pub fn get_position_ms(&self) -> i64 {
        self.position_ms.load(Ordering::SeqCst)
    }

    pub fn is_playing(&self) -> bool {
        self.is_playing.load(Ordering::SeqCst)
    }

    pub fn get_spectrum(&self) -> Vec<f32> {
        if self.is_playing() {
            self.spectrum.lock().compute_spectrum().to_vec()
        } else {
            vec![0.0; 32]
        }
    }
}
