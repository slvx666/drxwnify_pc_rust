use drxwnify_network::HttpClient;
use drxwnify_search::SearchService;

#[tokio::test]
async fn test_search_real() {
    let http = HttpClient::default();
    let service = SearchService::new(http);
    let results = service.search("Bring Me The Horizon Can You Feel My Heart").await.expect("search failed");
    println!("Found tracks: {}", results.tracks.len());
    for t in &results.tracks {
        println!("Track: {} by {:?} [id: {}, cover: {:?}]", t.title, t.artists, t.id, t.cover);
    }
    assert!(!results.tracks.is_empty(), "tracks should not be empty");
}
