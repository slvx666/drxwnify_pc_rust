use crate::queue::TrackQueue;
use crate::sleep_timer::SleepTimer;
use drxwnify_audio_engine::AudioEngine;
use drxwnify_core::{CoreEvent, DrxwnifyResult, EventBus};
use drxwnify_network::HttpClient;
use drxwnify_shared_models::*;
use drxwnify_stream_resolver::{PickMode, StreamResolver};
use parking_lot::Mutex;
use std::sync::Arc;

pub struct Player {
    audio_engine: Arc<AudioEngine>,
    event_bus: EventBus,
    queue: Arc<Mutex<TrackQueue>>,
    sleep_timer: Arc<Mutex<SleepTimer>>,
    resolver: Arc<StreamResolver>,
    http: HttpClient,
}

impl Player {
    pub fn new(audio_engine: Arc<AudioEngine>, event_bus: EventBus, http: HttpClient) -> Self {
        let resolver = Arc::new(StreamResolver::new(event_bus.clone(), http.clone(), None));
        Self {
            audio_engine,
            event_bus,
            queue: Arc::new(Mutex::new(TrackQueue::new())),
            sleep_timer: Arc::new(Mutex::new(SleepTimer::new())),
            resolver,
            http,
        }
    }

    pub fn play(&self) {
        self.audio_engine.play();
    }

    pub fn pause(&self) {
        self.audio_engine.pause();
    }

    pub fn stop(&self) {
        self.audio_engine.stop();
    }

    pub fn toggle_play_pause(&self) {
        if self.audio_engine.is_playing() {
            self.pause();
        } else {
            self.play();
        }
    }

    pub fn seek(&self, position_ms: i64) {
        let _ = self.audio_engine.seek(position_ms);
    }

    pub fn set_volume(&self, volume: f32) {
        self.audio_engine.set_volume(volume);
    }

    pub fn set_balance(&self, balance: f32) {
        self.audio_engine.set_balance(balance);
    }

    pub fn play_track(&self, track: Track) {
        {
            let mut q = self.queue.lock();
            q.set_tracks(vec![track.clone()], 0);
        }
        self.event_bus.emit(CoreEvent::TrackChanged(TrackChangedEvent {
            track: Some(track.clone()),
            index: 0,
            total: 1,
        }));
        self.load_and_play_track_async(track);
    }

    pub fn play_tracks(&self, tracks: Vec<Track>, start_index: usize) {
        let current_track = tracks.get(start_index).cloned();
        let total = tracks.len();
        {
            let mut q = self.queue.lock();
            q.set_tracks(tracks, start_index);
        }
        self.event_bus.emit(CoreEvent::TrackChanged(TrackChangedEvent {
            track: current_track.clone(),
            index: start_index,
            total,
        }));
        if let Some(t) = current_track {
            self.load_and_play_track_async(t);
        }
    }

    fn load_and_play_track_async(&self, track: Track) {
        let engine = Arc::clone(&self.audio_engine);
        let resolver = Arc::clone(&self.resolver);
        let http = self.http.clone();

        drxwnify_core::spawn_async(async move {
            tracing::info!("Resolving stream for track: {} - {}", track.title, track.artist_names());
            match resolver.resolve(&track, PickMode::Speed).await {
                Ok(stream) => {
                    tracing::info!("Stream resolved from {}: {} kbps", stream.source, stream.bitrate_kbps);
                    match Self::download_stream_bytes(&http, &stream).await {
                        Ok(bytes) => {
                            tracing::info!("Downloaded audio: {} bytes", bytes.len());
                            if let Err(e) = engine.load_audio_bytes(bytes) {
                                tracing::error!("AudioEngine failed to load audio bytes: {e}");
                            }
                        }
                        Err(e) => {
                            tracing::error!("Failed to download stream audio bytes: {e}");
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to resolve stream for {}: {e}", track.title);
                }
            }
        });
    }

    async fn download_stream_bytes(http: &HttpClient, stream: &ResolvedStream) -> Result<Vec<u8>, String> {
        if stream.url.contains(".m3u8") {
            let mut req = http.inner().get(&stream.url);
            for (k, v) in &stream.headers {
                req = req.header(k, v);
            }
            let playlist = req.send().await
                .map_err(|e| format!("Failed to fetch m3u8 playlist: {e}"))?
                .text().await
                .map_err(|e| format!("Failed to read m3u8 playlist text: {e}"))?;

            let mut audio_bytes = Vec::new();

            // 1. Check for fMP4 init map: #EXT-X-MAP:URI="init.mp4..."
            let init_re = regex::Regex::new(r#"#EXT-X-MAP:URI="([^"]+)""#).unwrap();
            if let Some(cap) = init_re.captures(&playlist) {
                let init_url = &cap[1];
                let mut init_req = http.inner().get(init_url);
                for (k, v) in &stream.headers {
                    init_req = init_req.header(k, v);
                }
                if let Ok(resp) = init_req.send().await {
                    if let Ok(data) = resp.bytes().await {
                        audio_bytes.extend_from_slice(&data);
                    }
                }
            }

            // 2. Extract media segment URLs
            let base_url = url::Url::parse(&stream.url).ok();
            for line in playlist.lines() {
                let trimmed = line.trim();
                if trimmed.is_empty() || trimmed.starts_with('#') {
                    continue;
                }
                let seg_url = if trimmed.starts_with("http") {
                    trimmed.to_string()
                } else if let Some(ref base) = base_url {
                    base.join(trimmed).map(|u| u.to_string()).unwrap_or_default()
                } else {
                    continue;
                };

                if !seg_url.is_empty() {
                    let mut s_req = http.inner().get(&seg_url);
                    for (k, v) in &stream.headers {
                        s_req = s_req.header(k, v);
                    }
                    if let Ok(resp) = s_req.send().await {
                        if let Ok(data) = resp.bytes().await {
                            audio_bytes.extend_from_slice(&data);
                        }
                    }
                }
            }

            if audio_bytes.is_empty() {
                return Err("Zero audio bytes assembled from m3u8 segments".to_string());
            }

            // If the assembled payload is MPEG-TS (starts with 0x47 sync byte),
            // extract the elementary audio stream (AAC ADTS / MP3) for Symphonia.
            if audio_bytes.first() == Some(&0x47) {
                let demuxed = extract_audio_from_ts(&audio_bytes);
                if !demuxed.is_empty() {
                    audio_bytes = demuxed;
                }
            }

            Ok(audio_bytes)
        } else {
            let mut req = http.inner().get(&stream.url);
            for (k, v) in &stream.headers {
                req = req.header(k, v);
            }
            let resp = req.send().await
                .map_err(|e| format!("GET {} failed: {e}", stream.url))?;
            let bytes = resp.bytes().await
                .map_err(|e| format!("Reading bytes from {} failed: {e}", stream.url))?;
            Ok(bytes.to_vec())
        }
    }

    pub fn next(&self) {
        let next_track = {
            let mut q = self.queue.lock();
            q.next().cloned()
        };
        let (idx, tot) = {
            let q = self.queue.lock();
            (q.current_index(), q.len())
        };
        self.event_bus.emit(CoreEvent::TrackChanged(TrackChangedEvent {
            track: next_track.clone(),
            index: idx,
            total: tot,
        }));
        if let Some(t) = next_track {
            self.load_and_play_track_async(t);
        }
    }

    pub fn previous(&self) {
        let prev_track = {
            let mut q = self.queue.lock();
            q.previous().cloned()
        };
        let (idx, tot) = {
            let q = self.queue.lock();
            (q.current_index(), q.len())
        };
        self.event_bus.emit(CoreEvent::TrackChanged(TrackChangedEvent {
            track: prev_track.clone(),
            index: idx,
            total: tot,
        }));
        if let Some(t) = prev_track {
            self.load_and_play_track_async(t);
        }
    }

    pub fn shuffle(&self) {
        self.queue.lock().shuffle();
    }

    pub fn set_repeat_mode(&self, mode: RepeatMode) {
        self.queue.lock().set_repeat_mode(mode);
    }

    pub fn repeat_mode(&self) -> RepeatMode {
        self.queue.lock().repeat_mode()
    }

    pub fn queue(&self) -> Arc<Mutex<TrackQueue>> {
        Arc::clone(&self.queue)
    }

    pub fn current_track(&self) -> Option<Track> {
        self.queue.lock().current_track().cloned()
    }

    pub fn is_playing(&self) -> bool {
        self.audio_engine.is_playing()
    }

    pub fn get_position_ms(&self) -> i64 {
        self.audio_engine.get_position_ms()
    }

    pub fn get_spectrum(&self) -> Vec<f32> {
        self.audio_engine.get_spectrum()
    }

    pub fn queue_tracks(&self) -> Vec<Track> {
        self.queue.lock().tracks().to_vec()
    }

    pub fn current_index(&self) -> usize {
        self.queue.lock().current_index()
    }

    pub fn clear_queue(&self) {
        self.queue.lock().clear();
    }

    pub fn play_queue_index(&self, index: usize) {
        let track = {
            let mut q = self.queue.lock();
            if index < q.len() {
                q.set_current_index(index);
                q.current_track().cloned()
            } else {
                None
            }
        };
        if let Some(t) = track {
            let total = self.queue.lock().len();
            self.event_bus.emit(CoreEvent::TrackChanged(TrackChangedEvent {
                track: Some(t.clone()),
                index,
                total,
            }));
            self.load_and_play_track_async(t);
        }
    }
}

pub fn extract_audio_from_ts(ts_data: &[u8]) -> Vec<u8> {
    let mut audio_es = Vec::new();
    let mut audio_pid: Option<u16> = None;
    let mut pmt_pid: Option<u16> = None;

    let mut offset = 0;
    while offset + 188 <= ts_data.len() {
        let packet = &ts_data[offset..offset + 188];
        offset += 188;

        if packet[0] != 0x47 {
            continue;
        }

        let pusi = (packet[1] & 0x40) != 0;
        let pid = (((packet[1] & 0x1F) as u16) << 8) | (packet[2] as u16);
        let afc = (packet[3] >> 4) & 0x03;

        let payload_start = match afc {
            0b01 => 4,
            0b11 => {
                let adapt_len = packet[4] as usize;
                5 + adapt_len
            }
            _ => continue,
        };

        if payload_start >= 188 {
            continue;
        }

        let payload = &packet[payload_start..];

        // 1. PAT (PID 0)
        if pid == 0 {
            let p_start = if pusi { 1 + payload[0] as usize } else { 0 };
            if p_start + 8 < payload.len() {
                let pat = &payload[p_start..];
                if pat[0] == 0x00 {
                    let section_len = (((pat[1] & 0x0F) as usize) << 8) | (pat[2] as usize);
                    let mut i = 8;
                    while i + 4 <= section_len && i + 4 <= pat.len() {
                        let prog_num = ((pat[i] as u16) << 8) | (pat[i + 1] as u16);
                        let prog_pid = (((pat[i + 2] & 0x1F) as u16) << 8) | (pat[i + 3] as u16);
                        if prog_num != 0 {
                            pmt_pid = Some(prog_pid);
                            break;
                        }
                        i += 4;
                    }
                }
            }
            continue;
        }

        // 2. PMT
        if Some(pid) == pmt_pid {
            let p_start = if pusi { 1 + payload[0] as usize } else { 0 };
            if p_start + 12 < payload.len() {
                let pmt = &payload[p_start..];
                if pmt[0] == 0x02 {
                    let section_len = (((pmt[1] & 0x0F) as usize) << 8) | (pmt[2] as usize);
                    let prog_info_len = (((pmt[10] & 0x0F) as usize) << 8) | (pmt[11] as usize);
                    let mut i = 12 + prog_info_len;
                    while i + 5 <= section_len && i + 5 <= pmt.len() {
                        let stream_type = pmt[i];
                        let elem_pid = (((pmt[i + 1] & 0x1F) as u16) << 8) | (pmt[i + 2] as u16);
                        let es_info_len = (((pmt[i + 3] & 0x0F) as usize) << 8) | (pmt[i + 4] as usize);
                        if stream_type == 0x0F || stream_type == 0x03 || stream_type == 0x04 || stream_type == 0x11 {
                            audio_pid = Some(elem_pid);
                            break;
                        }
                        i += 5 + es_info_len;
                    }
                }
            }
            continue;
        }

        // 3. Audio PES
        let is_audio = Some(pid) == audio_pid || (audio_pid.is_none() && pid >= 0x0100 && pid < 0x1FFF);
        if is_audio {
            if pusi {
                if payload.len() >= 9 && payload[0] == 0x00 && payload[1] == 0x00 && payload[2] == 0x01 {
                    let stream_id = payload[3];
                    if stream_id >= 0xC0 && stream_id <= 0xDF {
                        if audio_pid.is_none() {
                            audio_pid = Some(pid);
                        }
                        let header_data_len = payload[8] as usize;
                        let es_start = 9 + header_data_len;
                        if es_start < payload.len() {
                            audio_es.extend_from_slice(&payload[es_start..]);
                        }
                    }
                }
            } else if Some(pid) == audio_pid {
                audio_es.extend_from_slice(payload);
            }
        }
    }

    audio_es
}
