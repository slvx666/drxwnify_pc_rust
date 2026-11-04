use crate::queue::TrackQueue;
use crate::sleep_timer::SleepTimer;
use drxwnify_audio_engine::AudioEngine;
use drxwnify_core::{CoreEvent, EventBus};
use drxwnify_shared_models::*;
use parking_lot::Mutex;
use std::sync::Arc;

pub struct Player {
    audio_engine: Arc<AudioEngine>,
    event_bus: EventBus,
    queue: Arc<Mutex<TrackQueue>>,
    sleep_timer: Arc<Mutex<SleepTimer>>,
}

impl Player {
    pub fn new(audio_engine: Arc<AudioEngine>, event_bus: EventBus) -> Self {
        Self {
            audio_engine,
            event_bus,
            queue: Arc::new(Mutex::new(TrackQueue::new())),
            sleep_timer: Arc::new(Mutex::new(SleepTimer::new())),
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
            track: Some(track),
            index: 0,
            total: 1,
        }));
        self.audio_engine.play();
    }

    pub fn play_tracks(&self, tracks: Vec<Track>, start_index: usize) {
        let current_track = tracks.get(start_index).cloned();
        let total = tracks.len();
        {
            let mut q = self.queue.lock();
            q.set_tracks(tracks, start_index);
        }
        self.event_bus.emit(CoreEvent::TrackChanged(TrackChangedEvent {
            track: current_track,
            index: start_index,
            total,
        }));
        self.audio_engine.play();
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
            track: next_track,
            index: idx,
            total: tot,
        }));
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
            track: prev_track,
            index: idx,
            total: tot,
        }));
    }

    pub fn shuffle(&self) {
        self.queue.lock().shuffle();
    }

    pub fn set_repeat_mode(&self, mode: RepeatMode) {
        self.queue.lock().set_repeat_mode(mode);
    }

    pub fn queue(&self) -> Arc<Mutex<TrackQueue>> {
        Arc::clone(&self.queue)
    }

    pub fn current_track(&self) -> Option<Track> {
        self.queue.lock().current_track().cloned()
    }
}
