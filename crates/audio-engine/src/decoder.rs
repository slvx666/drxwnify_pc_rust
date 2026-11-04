use drxwnify_core::{DrxwnifyError, DrxwnifyResult};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum AudioSource {
    RemoteUrl {
        url: String,
        headers: Vec<(String, String)>,
    },
    LocalFile {
        path: PathBuf,
    },
}

pub struct AudioDecoder {
    pub source: AudioSource,
    pub sample_rate: u32,
    pub channels: u16,
    pub duration_ms: i64,
    pub current_position_ms: i64,
    pub is_eof: bool,
}

impl AudioDecoder {
    pub fn open(source: AudioSource) -> DrxwnifyResult<Self> {
        // Initializes decoder state for 48kHz S16 Stereo
        Ok(Self {
            source,
            sample_rate: 48000,
            channels: 2,
            duration_ms: 0,
            current_position_ms: 0,
            is_eof: false,
        })
    }

    pub fn attach_local_file(&mut self, path: PathBuf) {
        self.source = AudioSource::LocalFile { path };
    }

    pub fn seek(&mut self, position_ms: i64) -> DrxwnifyResult<()> {
        self.current_position_ms = position_ms;
        self.is_eof = false;
        Ok(())
    }

    pub fn decode_next_chunk(&mut self, max_frames: usize) -> DrxwnifyResult<Vec<(i16, i16)>> {
        if self.is_eof {
            return Ok(Vec::new());
        }

        // Return empty chunk when no active frames yet
        let chunk = Vec::with_capacity(max_frames);
        Ok(chunk)
    }
}
