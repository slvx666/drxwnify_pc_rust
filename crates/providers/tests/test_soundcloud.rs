use drxwnify_network::HttpClient;
use drxwnify_providers::SoundCloudProvider;
use drxwnify_shared_models::Track;

#[tokio::test]
async fn test_soundcloud_search_and_resolve() {
    let http = HttpClient::default();
    let mut sc = SoundCloudProvider::new(http);
    
    let track = Track::new("bmth:1".to_string(), "Can You Feel My Heart".to_string(), "Bring Me The Horizon".to_string(), 227);
    let res = sc.search_track(&track).await;
    println!("SoundCloud Stream Result: {:?}", res);
    assert!(res.is_ok(), "SoundCloud stream resolution should succeed: {:?}", res.err());
    let stream = res.unwrap();
    println!("Resolved SoundCloud stream: url={}, bitrate={}", stream.url, stream.bitrate_kbps);
    assert!(stream.url.starts_with("http"), "Should have valid HTTP stream url");
}
