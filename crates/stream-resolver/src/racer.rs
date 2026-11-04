use drxwnify_core::{CoreEvent, DrxwnifyError, DrxwnifyResult, EventBus};
use drxwnify_network::HttpClient;
use drxwnify_providers::*;
use drxwnify_shared_models::*;
use futures::stream::FuturesUnordered;
use futures::StreamExt;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PickMode {
    Speed,
    Quality,
    Patient,
}

pub struct StreamResolver {
    event_bus: EventBus,
    innertube: Arc<InnerTubeProvider>,
    vk: Arc<VkSourceProvider>,
    lossless: Arc<LosslessMirrorProvider>,
    bandcamp: Arc<BandcampProvider>,
}

impl StreamResolver {
    pub fn new(event_bus: EventBus, http: HttpClient, vk_token: Option<String>) -> Self {
        Self {
            event_bus,
            innertube: Arc::new(InnerTubeProvider::new(http.clone())),
            vk: Arc::new(VkSourceProvider::new(http.clone(), vk_token)),
            lossless: Arc::new(LosslessMirrorProvider::new(http.clone())),
            bandcamp: Arc::new(BandcampProvider::new(http)),
        }
    }

    pub async fn resolve(&self, track: &Track, mode: PickMode) -> DrxwnifyResult<ResolvedStream> {
        let max_wait = match mode {
            PickMode::Speed => Duration::from_millis(700),
            PickMode::Quality => Duration::from_millis(2000),
            PickMode::Patient => Duration::from_millis(8000),
        };

        let mut racers = FuturesUnordered::new();

        // 1. YouTube Racer
        let yt = Arc::clone(&self.innertube);
        let t_yt = track.clone();
        racers.push(tokio::spawn(async move {
            yt.resolve_track(&t_yt).await
        }));

        // 2. VK Racer
        let vk = Arc::clone(&self.vk);
        let t_vk = track.clone();
        racers.push(tokio::spawn(async move {
            vk.search_track(&t_vk).await
        }));

        // 3. Lossless Racer (if ISRC exists)
        if track.isrc.is_some() {
            let ll = Arc::clone(&self.lossless);
            let t_ll = track.clone();
            racers.push(tokio::spawn(async move {
                ll.resolve_track(&t_ll).await
            }));
        }

        let mut best_stream: Option<ResolvedStream> = None;

        while let Ok(Some(join_res)) = timeout(max_wait, racers.next()).await {
            if let Ok(Ok(stream)) = join_res {
                let current_bitrate = best_stream.as_ref().map(|s| s.bitrate_kbps).unwrap_or(0);
                if stream.bitrate_kbps > current_bitrate {
                    best_stream = Some(stream);
                    if mode == PickMode::Speed {
                        break;
                    }
                }
            }
        }

        if let Some(stream) = best_stream {
            Ok(stream)
        } else {
            // Last resort: await youtube fallback
            self.innertube.resolve_track(track).await
        }
    }
}
