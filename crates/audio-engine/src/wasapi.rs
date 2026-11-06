use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, Stream, StreamConfig};
use drxwnify_core::{DrxwnifyError, DrxwnifyResult};
use parking_lot::Mutex;
use std::collections::VecDeque;
use std::sync::Arc;

pub struct WasapiOutput {
    _stream: Option<Stream>,
    buffer: Arc<Mutex<VecDeque<(i16, i16)>>>,
    sample_rate: u32,
    channels: u16,
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

        let default_config = device
            .default_output_config()
            .map_err(|e| DrxwnifyError::Audio(format!("Failed to query default audio config: {e}")))?;

        let sample_format = default_config.sample_format();
        let stream_config: StreamConfig = default_config.into();
        let native_sample_rate = stream_config.sample_rate.0;
        let native_channels = stream_config.channels;

        tracing::info!(
            "Initializing WASAPI audio: rate={} Hz, channels={}, format={:?}",
            native_sample_rate,
            native_channels,
            sample_format
        );

        let buffer = Arc::new(Mutex::new(VecDeque::with_capacity(native_sample_rate as usize * 2)));
        let buffer_clone = Arc::clone(&buffer);

        let err_fn = |err| tracing::error!("WASAPI stream error: {}", err);

        let step = 48000.0 / native_sample_rate as f64;
        let mut sample_pos = 0.0f64;
        let mut last_l = 0.0f32;
        let mut last_r = 0.0f32;
        let mut next_l = 0.0f32;
        let mut next_r = 0.0f32;

        let stream = match sample_format {
            SampleFormat::F32 => device.build_output_stream(
                &stream_config,
                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    let mut buf = buffer_clone.lock();
                    let channels = native_channels as usize;

                    for frame in data.chunks_mut(channels) {
                        while sample_pos >= 1.0 {
                            sample_pos -= 1.0;
                            last_l = next_l;
                            last_r = next_r;
                            if let Some((l, r)) = buf.pop_front() {
                                next_l = l as f32 / 32768.0;
                                next_r = r as f32 / 32768.0;
                            } else {
                                next_l = 0.0;
                                next_r = 0.0;
                            }
                        }

                        let frac = sample_pos as f32;
                        let out_l = last_l + (next_l - last_l) * frac;
                        let out_r = last_r + (next_r - last_r) * frac;

                        frame[0] = out_l;
                        if channels > 1 {
                            frame[1] = out_r;
                        }
                        for c in 2..channels {
                            frame[c] = (out_l + out_r) * 0.5;
                        }

                        sample_pos += step;
                    }
                },
                err_fn,
                None,
            ),
            SampleFormat::I16 => device.build_output_stream(
                &stream_config,
                move |data: &mut [i16], _: &cpal::OutputCallbackInfo| {
                    let mut buf = buffer_clone.lock();
                    let channels = native_channels as usize;

                    for frame in data.chunks_mut(channels) {
                        while sample_pos >= 1.0 {
                            sample_pos -= 1.0;
                            last_l = next_l;
                            last_r = next_r;
                            if let Some((l, r)) = buf.pop_front() {
                                next_l = l as f32;
                                next_r = r as f32;
                            } else {
                                next_l = 0.0;
                                next_r = 0.0;
                            }
                        }

                        let frac = sample_pos as f32;
                        let out_l = (last_l + (next_l - last_l) * frac).clamp(-32768.0, 32767.0) as i16;
                        let out_r = (last_r + (next_r - last_r) * frac).clamp(-32768.0, 32767.0) as i16;

                        frame[0] = out_l;
                        if channels > 1 {
                            frame[1] = out_r;
                        }
                        for c in 2..channels {
                            frame[c] = ((out_l as i32 + out_r as i32) / 2) as i16;
                        }

                        sample_pos += step;
                    }
                },
                err_fn,
                None,
            ),
            _ => {
                return Err(DrxwnifyError::Audio(format!(
                    "Unsupported sample format: {:?}",
                    sample_format
                )))
            }
        }
        .map_err(|e| DrxwnifyError::Audio(format!("Failed to build WASAPI stream: {e}")))?;

        stream
            .play()
            .map_err(|e| DrxwnifyError::Audio(format!("Failed to play WASAPI stream: {e}")))?;

        Ok(Self {
            _stream: Some(stream),
            buffer,
            sample_rate: native_sample_rate,
            channels: native_channels,
        })
    }

    pub fn write_samples(&self, samples: &[(i16, i16)]) {
        let mut buf = self.buffer.lock();
        if buf.len() > 96000 {
            buf.drain(..48000);
        }
        buf.extend(samples.iter().copied());
    }

    pub fn clear(&self) {
        let mut buf = self.buffer.lock();
        buf.clear();
    }

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    pub fn channels(&self) -> u16 {
        self.channels
    }
}
