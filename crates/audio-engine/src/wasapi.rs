use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, Stream, StreamConfig};
use drxwnify_core::{DrxwnifyError, DrxwnifyResult};
use parking_lot::Mutex;
use std::collections::VecDeque;
use std::sync::Arc;

pub struct WasapiOutput {
    _stream: Option<Stream>,
    buffer: Arc<Mutex<VecDeque<(i16, i16)>>>,
}

unsafe impl Send for WasapiOutput {}
unsafe impl Sync for WasapiOutput {}

impl WasapiOutput {
    pub fn new(device_name: Option<&str>) -> DrxwnifyResult<Self> {
        let host = cpal::default_host();
        let device = match device_name {
            Some(name) => host
                .output_devices()
                .map_err(|e| DrxwnifyError::Audio(e.to_string()))?
                .find(|d| d.name().map(|n| n == name).unwrap_or(false))
                .unwrap_or(host.default_output_device().ok_or_else(|| {
                    DrxwnifyError::Audio("No audio output device found".to_string())
                })?),
            None => host.default_output_device().ok_or_else(|| {
                DrxwnifyError::Audio("No audio output device found".to_string())
            })?,
        };

        let buffer = Arc::new(Mutex::new(VecDeque::with_capacity(48000)));
        let buffer_clone = Arc::clone(&buffer);

        let config = StreamConfig {
            channels: 2,
            sample_rate: cpal::SampleRate(48000),
            buffer_size: cpal::BufferSize::Default,
        };

        let err_fn = |err| tracing::error!("WASAPI stream error: {}", err);

        let stream = device
            .build_output_stream(
                &config,
                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    let mut buf = buffer_clone.lock();
                    for frame in data.chunks_mut(2) {
                        if let Some((l, r)) = buf.pop_front() {
                            frame[0] = l as f32 / 32768.0;
                            if frame.len() > 1 {
                                frame[1] = r as f32 / 32768.0;
                            }
                        } else {
                            frame[0] = 0.0;
                            if frame.len() > 1 {
                                frame[1] = 0.0;
                            }
                        }
                    }
                },
                err_fn,
                None,
            )
            .map_err(|e| DrxwnifyError::Audio(format!("Failed to build WASAPI stream: {e}")))?;

        stream
            .play()
            .map_err(|e| DrxwnifyError::Audio(format!("Failed to play WASAPI stream: {e}")))?;

        Ok(Self {
            _stream: Some(stream),
            buffer,
        })
    }

    pub fn write_samples(&self, samples: &[(i16, i16)]) {
        let mut buf = self.buffer.lock();
        // Keep buffer bounded (~500ms max)
        if buf.len() > 48000 {
            buf.clear();
        }
        buf.extend(samples.iter().copied());
    }

    pub fn clear(&self) {
        let mut buf = self.buffer.lock();
        buf.clear();
    }
}
