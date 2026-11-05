use drxwnify_core::EventBus;
use drxwnify_network::HttpClient;
use drxwnify_shared_models::Track;
use drxwnify_stream_resolver::{PickMode, StreamResolver};

#[tokio::test]
async fn test_stream_resolver_racing() {
    let bus = EventBus::default();
    let http = HttpClient::default();
    let resolver = StreamResolver::new(bus, http, None);

    let track = Track::new("bmth:1".to_string(), "Can You Feel My Heart".to_string(), "Bring Me The Horizon".to_string(), 227);
    let res = resolver.resolve(&track, PickMode::Speed).await;
    println!("Resolver Race Result: {:?}", res);
    assert!(res.is_ok(), "StreamResolver should succeed: {:?}", res.err());
    let stream = res.unwrap();
    println!("Winner source: {}, bitrate: {} kbps, url: {}", stream.source, stream.bitrate_kbps, stream.url);
    assert!(!stream.url.is_empty(), "Stream URL should not be empty");
}
