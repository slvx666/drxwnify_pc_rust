pub mod dsp;
pub mod fft;
pub mod wasapi;
pub mod decoder;

pub use dsp::DspProcessor;
pub use fft::SpectrumAnalyzer;
pub use wasapi::WasapiOutput;
pub use decoder::{AudioSource, AudioDecoder};

use drxwnify_core::{CoreEvent, DrxwnifyError, DrxwnifyResult, EventBus};
use drxwnify_shared_models::*;
use parking_lot::Mutex;
use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};
use std::sync::Arc;
use std::time::Duration;

pub struct AudioEngine {
    event_bus: EventBus,
    output: Option<Arc<WasapiOutput>>,
    dsp: Arc<Mutex<DspProcessor>>,
    spectrum: Arc<Mutex<SpectrumAnalyzer>>,
    decoder: Arc<Mutex<Option<AudioDecoder>>>,
    is_playing: Arc<AtomicBool>,
    position_ms: Arc<AtomicI64>,
    duration_ms: Arc<AtomicI64>,
    is_running: Arc<AtomicBool>,
}

impl AudioEngine {
    pub fn new(event_bus: EventBus, device_name: Option<&str>) -> Self {
        let output = match WasapiOutput::new(device_name) {
            Ok(out) => Some(Arc::new(out)),
            Err(e) => {
                tracing::warn!("Could not initialize WASAPI output: {e}, falling back to silent engine");
                None
            }
        };

        let engine = Self {
            event_bus,
            output,
            dsp: Arc::new(Mutex::new(DspProcessor::new())),
            spectrum: Arc::new(Mutex::new(SpectrumAnalyzer::new())),
            decoder: Arc::new(Mutex::new(None)),
            is_playing: Arc::new(AtomicBool::new(false)),
            position_ms: Arc::new(AtomicI64::new(0)),
            duration_ms: Arc::new(AtomicI64::new(0)),
            is_running: Arc::new(AtomicBool::new(true)),
        };

        engine.start_playback_worker();
        engine
    }

    fn start_playback_worker(&self) {
        let decoder_arc = Arc::clone(&self.decoder);
        let output_opt = self.output.clone();
        let is_playing_arc = Arc::clone(&self.is_playing);
        let is_running_arc = Arc::clone(&self.is_running);
        let position_ms_arc = Arc::clone(&self.position_ms);
        let duration_ms_arc = Arc::clone(&self.duration_ms);
        let dsp_arc = Arc::clone(&self.dsp);
        let spectrum_arc = Arc::clone(&self.spectrum);
        let event_bus = self.event_bus.clone();

        std::thread::Builder::new()
            .name("drxwnify-audio-worker".to_string())
            .spawn(move || {
                let chunk_size = 960; // 20ms of audio at 48kHz
                while is_running_arc.load(Ordering::SeqCst) {
                    if !is_playing_arc.load(Ordering::SeqCst) {
                        std::thread::sleep(Duration::from_millis(15));
                        continue;
                    }

                    let decoded_chunk = {
                        let mut dec_guard = decoder_arc.lock();
                        if let Some(ref mut dec) = *dec_guard {
                            if dec.is_eof {
                                None
                            } else {
                                match dec.decode_next_chunk(chunk_size) {
                                    Ok(frames) if !frames.is_empty() => {
                                        position_ms_arc.store(dec.current_position_ms, Ordering::SeqCst);
                                        Some(frames)
                                    }
                                    _ => None,
                                }
                            }
                        } else {
                            None
                        }
                    };

                    match decoded_chunk {
                        Some(frames) => {
                            // Process DSP (volume, balance, normalize)
                            let mut processed = Vec::with_capacity(frames.len());
                            {
                                let mut dsp = dsp_arc.lock();
                                for &(l, r) in &frames {
                                    let (pl, pr) = dsp.process_stereo_frame(l, r);
                                    processed.push((pl, pr));
                                }
                            }

                            // Feed FFT spectrum analyzer
                            {
                                let mut spec = spectrum_arc.lock();
                                spec.push_samples(&processed);
                            }

                            // Feed WASAPI output
                            if let Some(ref out) = output_opt {
                                out.write_samples(&processed);
                            }

                            // Progress event emit throttle (~100ms)
                            let cur_pos = position_ms_arc.load(Ordering::SeqCst);
                            let total_dur_sec = duration_ms_arc.load(Ordering::SeqCst) / 1000;
                            event_bus.emit(CoreEvent::Progress(PlaybackProgressEvent {
                                position_ms: cur_pos,
                                duration_sec: total_dur_sec,
                                state: PlaybackState::Playing,
                            }));

                            // Buffer pacing: don't spin faster than real-time audio playback
                            std::thread::sleep(Duration::from_millis(16));
                        }
                        None => {
                            // Idle / waiting for track
                            std::thread::sleep(Duration::from_millis(20));
                        }
                    }
                }
            })
            .expect("Failed to start audio playback worker thread");
    }

    pub fn load_audio_bytes(&self, bytes: Vec<u8>) -> DrxwnifyResult<()> {
        let dec = AudioDecoder::open_memory(bytes)?;
        self.duration_ms.store(dec.duration_ms, Ordering::SeqCst);
        self.position_ms.store(0, Ordering::SeqCst);
        if let Some(ref out) = self.output {
            out.clear();
        }
        *self.decoder.lock() = Some(dec);
        self.play();
        Ok(())
    }

    pub fn load_local_file(&self, path: &std::path::Path) -> DrxwnifyResult<()> {
        let bytes = std::fs::read(path).map_err(|e| DrxwnifyError::Audio(e.to_string()))?;
        self.load_audio_bytes(bytes)
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
        *self.decoder.lock() = None;
        self.event_bus.emit(CoreEvent::StateChanged(PlaybackState::Stopped));
    }

    pub fn seek(&self, position_ms: i64) -> DrxwnifyResult<()> {
        self.position_ms.store(position_ms, Ordering::SeqCst);
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

    pub fn get_position_ms(&self) -> i64 {
        self.position_ms.load(Ordering::SeqCst)
    }

    pub fn is_playing(&self) -> bool {
        self.is_playing.load(Ordering::SeqCst)
    }

    pub fn get_spectrum(&self) -> Vec<f32> {
        self.spectrum.lock().compute_spectrum().to_vec()
    }
}

impl Drop for AudioEngine {
    fn drop(&mut self) {
        self.is_running.store(false, Ordering::SeqCst);
    }
}
