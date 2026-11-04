use drxwnify_shared_models::*;
use tokio::sync::broadcast;

#[derive(Debug, Clone)]
pub enum CoreEvent {
    Progress(PlaybackProgressEvent),
    TrackChanged(TrackChangedEvent),
    SourceUpgrade(SourceUpgradeEvent),
    Spectrum(SpectrumEvent),
    StateChanged(PlaybackState),
}

#[derive(Clone)]
pub struct EventBus {
    sender: broadcast::Sender<CoreEvent>,
}

impl EventBus {
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self { sender }
    }

    pub fn emit(&self, event: CoreEvent) {
        let _ = self.sender.send(event);
    }

    pub fn subscribe(&self) -> broadcast::Receiver<CoreEvent> {
        self.sender.subscribe()
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new(256)
    }
}
