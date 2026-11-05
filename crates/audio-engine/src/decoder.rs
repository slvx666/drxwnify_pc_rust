use drxwnify_core::{DrxwnifyError, DrxwnifyResult};
use std::io::Cursor;
use std::path::PathBuf;
use symphonia::core::audio::{AudioBufferRef, Signal};
use symphonia::core::codecs::{Decoder, DecoderOptions};
use symphonia::core::formats::{FormatOptions, FormatReader};
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

#[derive(Debug, Clone)]
pub enum AudioSource {
    RemoteUrl {
        url: String,
        headers: Vec<(String, String)>,
    },
    LocalFile {
        path: PathBuf,
    },
    Memory {
        bytes: Vec<u8>,
    },
}

pub struct AudioDecoder {
    format_reader: Box<dyn FormatReader>,
    decoder: Box<dyn Decoder>,
    track_id: u32,
    sample_rate: u32,
    channels: u16,
    pub duration_ms: i64,
    pub current_position_ms: i64,
    pub is_eof: bool,
    sample_buffer: Vec<(i16, i16)>,
}

// Safety: symphonia reader/decoder are used sequentially within the audio worker thread
unsafe impl Send for AudioDecoder {}
unsafe impl Sync for AudioDecoder {}

impl AudioDecoder {
    pub fn open_memory(bytes: Vec<u8>) -> DrxwnifyResult<Self> {
        let cursor = Cursor::new(bytes);
        let mss = MediaSourceStream::new(Box::new(cursor), Default::default());

        let hint = Hint::new();
        let format_opts = FormatOptions {
            enable_gapless: true,
            ..Default::default()
        };
        let metadata_opts = MetadataOptions::default();

        let probed = symphonia::default::get_probe()
            .format(&hint, mss, &format_opts, &metadata_opts)
            .map_err(|e| DrxwnifyError::Audio(format!("Probe error: {e}")))?;

        let format_reader = probed.format;
        let track = format_reader
            .tracks()
            .iter()
            .find(|t| t.codec_params.codec != symphonia::core::codecs::CODEC_TYPE_NULL)
            .ok_or_else(|| DrxwnifyError::Audio("No valid audio track found".to_string()))?;

        let track_id = track.id;
        let sample_rate = track.codec_params.sample_rate.unwrap_or(44100);
        let channels = track.codec_params.channels.map(|c| c.count() as u16).unwrap_or(2);

        let duration_ms = if let Some(n_frames) = track.codec_params.n_frames {
            (n_frames as f64 / sample_rate as f64 * 1000.0) as i64
        } else {
            0
        };

        let dec_opts = DecoderOptions::default();
        let decoder = symphonia::default::get_codecs()
            .make(&track.codec_params, &dec_opts)
            .map_err(|e| DrxwnifyError::Audio(format!("Codec error: {e}")))?;

        Ok(Self {
            format_reader,
            decoder,
            track_id,
            sample_rate,
            channels,
            duration_ms,
            current_position_ms: 0,
            is_eof: false,
            sample_buffer: Vec::with_capacity(8192),
        })
    }

    pub fn decode_next_chunk(&mut self, max_frames: usize) -> DrxwnifyResult<Vec<(i16, i16)>> {
        while self.sample_buffer.len() < max_frames && !self.is_eof {
            match self.format_reader.next_packet() {
                Ok(packet) => {
                    if packet.track_id() != self.track_id {
                        continue;
                    }
                    match self.decoder.decode(&packet) {
                        Ok(decoded) => {
                            Self::append_samples(&mut self.sample_buffer, &decoded);
                        }
                        Err(symphonia::core::errors::Error::DecodeError(msg)) => {
                            tracing::warn!("Decode warning: {msg}");
                        }
                        Err(e) => {
                            tracing::error!("Decode error: {e}");
                            self.is_eof = true;
                            break;
                        }
                    }
                }
                Err(symphonia::core::errors::Error::IoError(ref e))
                    if e.kind() == std::io::ErrorKind::UnexpectedEof =>
                {
                    self.is_eof = true;
                    break;
                }
                Err(e) => {
                    tracing::debug!("End of stream / packet: {e}");
                    self.is_eof = true;
                    break;
                }
            }
        }

        let count = max_frames.min(self.sample_buffer.len());
        let frames: Vec<(i16, i16)> = self.sample_buffer.drain(0..count).collect();

        if self.sample_rate > 0 {
            let frames_duration_ms = (frames.len() as f64 / self.sample_rate as f64 * 1000.0) as i64;
            self.current_position_ms += frames_duration_ms;
        }

        Ok(frames)
    }

    fn append_samples(sample_buffer: &mut Vec<(i16, i16)>, buffer: &AudioBufferRef) {
        match buffer {
            AudioBufferRef::F32(buf) => {
                let n_frames = buf.frames();
                let chan_count = buf.spec().channels.count();
                for i in 0..n_frames {
                    let l = (buf.chan(0)[i].clamp(-1.0, 1.0) * 32767.0) as i16;
                    let r = if chan_count > 1 {
                        (buf.chan(1)[i].clamp(-1.0, 1.0) * 32767.0) as i16
                    } else {
                        l
                    };
                    sample_buffer.push((l, r));
                }
            }
            AudioBufferRef::S16(buf) => {
                let n_frames = buf.frames();
                let chan_count = buf.spec().channels.count();
                for i in 0..n_frames {
                    let l = buf.chan(0)[i];
                    let r = if chan_count > 1 { buf.chan(1)[i] } else { l };
                    sample_buffer.push((l, r));
                }
            }
            AudioBufferRef::U8(buf) => {
                let n_frames = buf.frames();
                let chan_count = buf.spec().channels.count();
                for i in 0..n_frames {
                    let l = (buf.chan(0)[i] as i16 - 128) << 8;
                    let r = if chan_count > 1 { (buf.chan(1)[i] as i16 - 128) << 8 } else { l };
                    sample_buffer.push((l, r));
                }
            }
            _ => {}
        }
    }

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }
}
