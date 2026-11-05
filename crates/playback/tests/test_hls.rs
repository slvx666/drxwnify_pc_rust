use drxwnify_audio_engine::AudioDecoder;
use drxwnify_core::EventBus;
use drxwnify_network::HttpClient;
use drxwnify_shared_models::Track;
use drxwnify_stream_resolver::{PickMode, StreamResolver};

#[tokio::test]
async fn test_full_resolve_and_decode_pipeline() {
    let bus = EventBus::default();
    let http = HttpClient::default();
    let resolver = StreamResolver::new(bus, http.clone(), None);

    let track = Track::new("bmth:1".to_string(), "Can You Feel My Heart".to_string(), "Bring Me The Horizon".to_string(), 227);
    // Try SoundCloud resolution
    let mut sc = drxwnify_providers::SoundCloudProvider::new(http.clone());
    let stream = sc.search_track(&track).await.expect("SoundCloud must resolve stream");
    println!("Resolved SoundCloud stream: source={}, protocol={:?}, url={}", stream.source, stream.protocol, stream.url);

    let bytes = if stream.url.contains(".m3u8") {
        let playlist = http.inner().get(&stream.url).send().await.unwrap().text().await.unwrap();
        let mut audio_bytes = Vec::new();

        // 1. Check for #EXT-X-MAP:URI="init.mp4..."
        let init_re = regex::Regex::new(r#"#EXT-X-MAP:URI="([^"]+)""#).unwrap();
        if let Some(cap) = init_re.captures(&playlist) {
            let init_url = &cap[1];
            println!("Downloading init segment: {}", &init_url[..init_url.len().min(80)]);
            let init_data = http.inner().get(init_url).send().await.unwrap().bytes().await.unwrap();
            audio_bytes.extend_from_slice(&init_data);
        }

        // 2. Extract media segment URLs
        let mut count = 0;
        for line in playlist.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            if trimmed.starts_with("http") {
                if let Ok(resp) = http.inner().get(trimmed).send().await {
                    if let Ok(seg_data) = resp.bytes().await {
                        audio_bytes.extend_from_slice(&seg_data);
                        count += 1;
                        if count >= 3 {
                            break; // 3 segments (~30s) is plenty for the test
                        }
                    }
                }
            }
        }
        audio_bytes
    } else {
        http.inner().get(&stream.url).send().await.unwrap().bytes().await.unwrap().to_vec()
    };

    println!("Total audio bytes assembled: {}", bytes.len());
    assert!(bytes.len() > 10000, "Should have downloaded audio bytes");

    // Open in Symphonia decoder
    let mut decoder = AudioDecoder::open_memory(bytes).expect("Symphonia should decode assembled fMP4/MP3");
    println!("Decoder duration: {} ms, sample_rate: {}", decoder.duration_ms, decoder.sample_rate());
    assert!(decoder.sample_rate() > 0);

    // Decode first chunk
    let chunk = decoder.decode_next_chunk(960).expect("Should decode chunk");
    println!("Decoded frames count: {}", chunk.len());
    assert!(!chunk.is_empty());
}
