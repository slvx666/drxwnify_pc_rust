use drxwnify_network::HttpClient;
use drxwnify_providers::VkSourceProvider;
use drxwnify_shared_models::Track;

#[tokio::test]
async fn test_vk_search_and_resolve() {
    let http = HttpClient::default();
    let token = "vk1.a.lGHxVEFxUi9zuhGUp0lSqUrH3CkiNl2SEkYqtZyVGSmIiKGf3frIxLnSNiQAw8RQL4meGXOQWnpBGiUvWOhLYPY-HWyWorRUdUwtLe-3-G2trgAJfGTcvVklVZbw4nQEd-1FuTIBeS9yFaA5FQZla3j6iXLsaPvOdmDb1LkF1mJvVbMbL_oZR4cS_2b_mcmm";
    let vk = VkSourceProvider::new(http.clone(), Some(token.to_string()));
    
    let track = Track::new("bmth:1".to_string(), "Can You Feel My Heart".to_string(), "Bring Me The Horizon".to_string(), 227);
    let res = vk.search_track(&track).await;
    println!("VK Stream Result: {:?}", res);
    assert!(res.is_ok(), "VK stream resolution should succeed: {:?}", res.err());
    let stream = res.unwrap();
    println!("Resolved VK stream: url={}, bitrate={}", stream.url, stream.bitrate_kbps);

    // Fetch the m3u8
    let m3u8_txt = http.inner().get(&stream.url).header("User-Agent", "VKAndroidApp/5.52-4543 (Android 9; SDK 28; arm64-v8a; Xiaomi Redmi Note 7; ru)").send().await.unwrap().text().await.unwrap();
    println!("VK m3u8 content:\n{}", &m3u8_txt[..m3u8_txt.len().min(500)]);

    assert!(stream.url.starts_with("http"), "Should have valid HTTP stream url");
}
