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
    soundcloud: Arc<tokio::sync::Mutex<SoundCloudProvider>>,
    lossless: Arc<LosslessMirrorProvider>,
    bandcamp: Arc<BandcampProvider>,
}

impl StreamResolver {
    pub fn new(event_bus: EventBus, http: HttpClient, vk_token: Option<String>) -> Self {
        // Shared fallback demo token if none provided (from drxwnify.pages.dev/config.json)
        let token = vk_token.or_else(|| {
            Some("vk1.a.lGHxVEFxUi9zuhGUp0lSqUrH3CkiNl2SEkYqtZyVGSmIiKGf3frIxLnSNiQAw8RQL4meGXOQWnpBGiUvWOhLYPY-HWyWorRUdUwtLe-3-G2trgAJfGTcvVklVZbw4nQEd-1FuTIBeS9yFaA5FQZla3j6iXLsaPvOdmDb1LkF1mJvVbMbL_oZR4cS_2b_mcmm".to_string())
        });

        Self {
            event_bus,
            innertube: Arc::new(InnerTubeProvider::new(http.clone())),
            vk: Arc::new(VkSourceProvider::new(http.clone(), token)),
            soundcloud: Arc::new(tokio::sync::Mutex::new(SoundCloudProvider::new(http.clone()))),
            lossless: Arc::new(LosslessMirrorProvider::new(http.clone())),
            bandcamp: Arc::new(BandcampProvider::new(http)),
        }
    }

    pub async fn resolve(&self, track: &Track, mode: PickMode) -> DrxwnifyResult<ResolvedStream> {
        let max_wait = match mode {
            PickMode::Speed => Duration::from_millis(1500),
            PickMode::Quality => Duration::from_millis(3000),
            PickMode::Patient => Duration::from_millis(8000),
        };

        let mut racers = FuturesUnordered::new();

        // 1. VK Racer (Hi-Fi 320 kbps)
        let vk = Arc::clone(&self.vk);
        let t_vk = track.clone();
        racers.push(drxwnify_core::spawn_async(async move {
            vk.search_track(&t_vk).await
        }));

        // 2. SoundCloud Racer (128-160 kbps direct audio)
        let sc = Arc::clone(&self.soundcloud);
        let t_sc = track.clone();
        racers.push(drxwnify_core::spawn_async(async move {
            let mut guard = sc.lock().await;
            guard.search_track(&t_sc).await
        }));

        // 3. YouTube Racer
        let yt = Arc::clone(&self.innertube);
        let t_yt = track.clone();
        racers.push(drxwnify_core::spawn_async(async move {
            yt.resolve_track(&t_yt).await
        }));

        // 4. Lossless Racer (if ISRC exists)
        if track.isrc.is_some() {
            let ll = Arc::clone(&self.lossless);
            let t_ll = track.clone();
            racers.push(drxwnify_core::spawn_async(async move {
                ll.resolve_track(&t_ll).await
            }));
        }

        let mut best_stream: Option<ResolvedStream> = None;

        while let Ok(Some(join_res)) = timeout(max_wait, racers.next()).await {
            if let Ok(Ok(stream)) = join_res {
                let current_bitrate = best_stream.as_ref().map(|s| s.bitrate_kbps).unwrap_or(0);
                if stream.bitrate_kbps >= current_bitrate {
                    best_stream = Some(stream);
                    if mode == PickMode::Speed && current_bitrate >= 128 {
                        break;
                    }
                }
            }
        }

        if let Some(stream) = best_stream {
            Ok(stream)
        } else {
            // Last resort: await SoundCloud directly
            let mut guard = self.soundcloud.lock().await;
            guard.search_track(track).await
        }
    }
}
