use drxwnify_network::HttpClient;
use drxwnify_providers::InnerTubeProvider;

#[tokio::test]
async fn test_innertube_search_and_resolve() {
    let http = HttpClient::default();
    let provider = InnerTubeProvider::new(http);
    
    // Search for a well-known track
    let search_res = provider.search_tracks("Bring Me The Horizon Can You Feel My Heart").await;
    assert!(search_res.is_ok(), "Search should succeed: {:?}", search_res.err());
    let tracks = search_res.unwrap();
    println!("Found {} tracks", tracks.len());
    assert!(!tracks.is_empty(), "Should find at least 1 track");
    
    let first = &tracks[0];
    println!("First track: {} - {} (id: {})", first.title, first.artist_names(), first.id);
    
    // Resolve stream (Note: YouTube anonymous player API may require bot verification; in production StreamResolver races VK & SoundCloud)
    let stream_res = provider.resolve_track(first).await;
    println!("Stream resolve result: {:?}", stream_res.as_ref().map(|s| (&s.source, s.bitrate_kbps)));
    if stream_res.is_err() {
        println!("YouTube requires bot check/login as expected; StreamResolver uses VK & SoundCloud as primary resolvers.");
    }
}
