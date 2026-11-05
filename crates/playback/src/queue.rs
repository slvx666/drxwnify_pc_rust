use drxwnify_shared_models::{RepeatMode, Track};
use rand::seq::SliceRandom;
use rand::thread_rng;

pub const MAX_QUEUE_SIZE: usize = 500;

#[derive(Debug, Clone, Default)]
pub struct TrackQueue {
    tracks: Vec<Track>,
    current_index: usize,
    repeat_mode: RepeatMode,
}

impl TrackQueue {
    pub fn new() -> Self {
        Self {
            tracks: Vec::with_capacity(MAX_QUEUE_SIZE),
            current_index: 0,
            repeat_mode: RepeatMode::Off,
        }
    }

    pub fn len(&self) -> usize {
        self.tracks.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tracks.is_empty()
    }

    pub fn current_track(&self) -> Option<&Track> {
        self.tracks.get(self.current_index)
    }

    pub fn current_index(&self) -> usize {
        self.current_index
    }

    pub fn set_current_index(&mut self, idx: usize) {
        if idx < self.tracks.len() {
            self.current_index = idx;
        }
    }

    pub fn clear(&mut self) {
        self.tracks.clear();
        self.current_index = 0;
    }

    pub fn repeat_mode(&self) -> RepeatMode {
        self.repeat_mode
    }

    pub fn set_repeat_mode(&mut self, mode: RepeatMode) {
        self.repeat_mode = mode;
    }

    pub fn tracks(&self) -> &[Track] {
        &self.tracks
    }

    pub fn set_tracks(&mut self, tracks: Vec<Track>, start_index: usize) {
        self.tracks = tracks;
        if self.tracks.len() > MAX_QUEUE_SIZE {
            self.tracks.truncate(MAX_QUEUE_SIZE);
        }
        self.current_index = if self.tracks.is_empty() { 0 } else { start_index.min(self.tracks.len() - 1) };
    }

    pub fn push_back(&mut self, track: Track) {
        if self.tracks.len() < MAX_QUEUE_SIZE {
            self.tracks.push(track);
        }
    }

    pub fn insert_next(&mut self, track: Track) {
        if self.tracks.is_empty() {
            self.tracks.push(track);
            self.current_index = 0;
        } else if self.tracks.len() < MAX_QUEUE_SIZE {
            self.tracks.insert(self.current_index + 1, track);
        }
    }

    pub fn remove(&mut self, index: usize) -> Option<Track> {
        if index >= self.tracks.len() {
            return None;
        }
        let removed = self.tracks.remove(index);
        if index < self.current_index {
            self.current_index -= 1;
        } else if self.current_index >= self.tracks.len() && !self.tracks.is_empty() {
            self.current_index = self.tracks.len() - 1;
        }
        Some(removed)
    }

    pub fn reorder(&mut self, from: usize, to: usize) {
        if from >= self.tracks.len() || to >= self.tracks.len() || from == to {
            return;
        }
        let track = self.tracks.remove(from);
        self.tracks.insert(to, track);

        if self.current_index == from {
            self.current_index = to;
        } else if from < self.current_index && to >= self.current_index {
            self.current_index -= 1;
        } else if from > self.current_index && to <= self.current_index {
            self.current_index += 1;
        }
    }

    pub fn shuffle(&mut self) {
        if self.tracks.len() <= 1 {
            return;
        }

        // Keep current track at index 0
        let current = self.tracks.remove(self.current_index);
        let mut rng = thread_rng();
        self.tracks.shuffle(&mut rng);
        self.tracks.insert(0, current);
        self.current_index = 0;
    }

    pub fn next(&mut self) -> Option<&Track> {
        if self.tracks.is_empty() {
            return None;
        }

        match self.repeat_mode {
            RepeatMode::One => self.current_track(),
            RepeatMode::All => {
                self.current_index = (self.current_index + 1) % self.tracks.len();
                self.current_track()
            }
            RepeatMode::Off => {
                if self.current_index + 1 < self.tracks.len() {
                    self.current_index += 1;
                    self.current_track()
                } else {
                    None
                }
            }
        }
    }

    pub fn previous(&mut self) -> Option<&Track> {
        if self.tracks.is_empty() {
            return None;
        }

        if self.current_index > 0 {
            self.current_index -= 1;
        } else if self.repeat_mode == RepeatMode::All {
            self.current_index = self.tracks.len() - 1;
        }

        self.current_track()
    }
}
