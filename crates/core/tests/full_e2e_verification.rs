use drxwnify_audio_engine::decoder::AudioDecoder;
use drxwnify_audio_engine::dsp::DspProcessor;
use drxwnify_audio_engine::fft::SpectrumAnalyzer;
use drxwnify_audio_engine::AudioEngine;
use drxwnify_core::paths::{AppEdition, AppPaths};
use drxwnify_core::EventBus;
use drxwnify_friends::{ChatHubManager, ChatMessage};
use drxwnify_library::LibraryManager;
use drxwnify_lyrics::LyricsService;
use drxwnify_network::HttpClient;
use drxwnify_nostr::NostrIdentity;
use drxwnify_persistence::Store;
use drxwnify_playback::Player;
use drxwnify_search::SearchService;
use drxwnify_shared_models::*;
use drxwnify_stream_resolver::{PickMode, StreamResolver};
use std::sync::Arc;

#[tokio::test]
async fn test_e2e_01_search_and_metadata() {
    println!("\n=== [E2E TEST 1] Search & Metadata Pipeline ===");
    let http = HttpClient::default();
    let search = SearchService::new(http);

    let results = search.search("Bring Me The Horizon Can You Feel My Heart").await.expect("Search must succeed");
    println!("Found tracks: {}", results.tracks.len());
    assert!(!results.tracks.is_empty(), "Search should find tracks");

    let first = &results.tracks[0];
    println!("First track: '{}' by {:?}", first.title, first.artists);
    assert!(!first.title.is_empty(), "Track title must not be empty");
    assert!(!first.artists.is_empty(), "Track must have at least one artist");
    assert!(first.id.starts_with("yt:"), "Track ID must have valid catalog prefix");
    println!("[E2E TEST 1] PASSED: Search and metadata verified.");
}

#[tokio::test]
async fn test_e2e_02_stream_resolution_and_racing() {
    println!("\n=== [E2E TEST 2] Stream Resolution & Source Racing ===");
    let http = HttpClient::default();
    let bus = EventBus::default();
    let token = "vk1.a.lGHxVEFxUi9zuhGUp0lSqUrH3CkiNl2SEkYqtZyVGSmIiKGf3frIxLnSNiQAw8RQL4meGXOQWnpBGiUvWOhLYPY-HWyWorRUdUwtLe-3-G2trgAJfGTcvVklVZbw4nQEd-1FuTIBeS9yFaA5FQZla3j6iXLsaPvOdmDb1LkF1mJvVbMbL_oZR4cS_2b_mcmm";
    let resolver = StreamResolver::new(bus, http, Some(token.to_string()));

    let track = Track::new("t1".to_string(), "Can You Feel My Heart".to_string(), "Bring Me The Horizon".to_string(), 227);
    let resolved = resolver.resolve(&track, PickMode::Speed).await.expect("Stream racing must resolve a winner");

    println!("Winner source: {}, bitrate: {} kbps, codec: {:?}", resolved.source, resolved.bitrate_kbps, resolved.codec);
    assert!(resolved.bitrate_kbps >= 128, "Bitrate must be at least 128 kbps");
    assert!(resolved.url.starts_with("http"), "Stream URL must be valid HTTP/HTTPS");
    println!("[E2E TEST 2] PASSED: Multi-source stream racing verified.");
}

#[tokio::test]
async fn test_e2e_03_audio_decoding_and_dsp_pipeline() {
    println!("\n=== [E2E TEST 3] Audio Decoding, MPEG-TS Demuxing & DSP ===");
    let http = HttpClient::default();
    let vk = drxwnify_providers::VkSourceProvider::new(http.clone(), Some("vk1.a.lGHxVEFxUi9zuhGUp0lSqUrH3CkiNl2SEkYqtZyVGSmIiKGf3frIxLnSNiQAw8RQL4meGXOQWnpBGiUvWOhLYPY-HWyWorRUdUwtLe-3-G2trgAJfGTcvVklVZbw4nQEd-1FuTIBeS9yFaA5FQZla3j6iXLsaPvOdmDb1LkF1mJvVbMbL_oZR4cS_2b_mcmm".to_string()));
    
    let track = Track::new("t1".to_string(), "Can You Feel My Heart".to_string(), "Bring Me The Horizon".to_string(), 227);
    let stream = vk.search_track(&track).await.expect("VK stream resolution");

    let base = url::Url::parse(&stream.url).unwrap();
    let seg_url = base.join("seg-00-a2.ts?siren=1").unwrap();
    let seg_bytes = http.inner().get(seg_url.as_str())
        .header("User-Agent", "VKAndroidApp/5.52-4543 (Android 9; SDK 28; arm64-v8a; Xiaomi Redmi Note 7; ru)")
        .send().await.unwrap().bytes().await.unwrap();

    let audio_data = drxwnify_playback::player::extract_audio_from_ts(&seg_bytes);
    assert!(!audio_data.is_empty(), "MPEG-TS extraction must yield audio ES");

    // Open Symphonia decoder
    let mut decoder = AudioDecoder::open_memory(audio_data).expect("Symphonia probe must succeed on demuxed audio");
    let sample_rate = decoder.sample_rate();
    println!("Decoded stream sample_rate: {} Hz", sample_rate);
    assert_eq!(sample_rate, 44100);

    let pcm_frames = decoder.decode_next_chunk(1024).expect("Decode next chunk");
    assert!(!pcm_frames.is_empty(), "Must decode at least 1 chunk");
    println!("Decoded frames in chunk: {}", pcm_frames.len());

    // Run through DSP
    let mut dsp = DspProcessor::new();
    dsp.volume = 0.8;
    dsp.balance = -0.2;

    let mut analyzer = SpectrumAnalyzer::new();
    let mut processed_frames = Vec::new();

    for &(l, r) in &pcm_frames {
        let (out_l, out_r) = dsp.process_stereo_frame(l, r);
        processed_frames.push((out_l, out_r));
    }

    analyzer.push_samples(&processed_frames);
    let spectrum = analyzer.compute_spectrum();
    assert_eq!(spectrum.len(), 32, "FFT Spectrum must have 32 bands");
    println!("[E2E TEST 3] PASSED: Audio decoding, demuxing, DSP and FFT verified.");
}

#[tokio::test]
async fn test_e2e_04_player_state_machine_and_queue() {
    println!("\n=== [E2E TEST 4] Player State Machine & Queue Management ===");
    let bus = EventBus::default();
    let engine = Arc::new(AudioEngine::new(bus.clone(), None));
    let http = HttpClient::default();
    let player = Player::new(engine, bus, http);

    let tracks = vec![
        Track::new("1", "Track 1", "Artist A", 180),
        Track::new("2", "Track 2", "Artist B", 200),
        Track::new("3", "Track 3", "Artist C", 210),
    ];

    player.play_tracks(tracks.clone(), 0);
    assert_eq!(player.current_index(), 0);
    assert_eq!(player.current_track().unwrap().title, "Track 1");
    assert_eq!(player.queue_tracks().len(), 3);

    // Test shuffle preserves first track
    player.shuffle();
    assert_eq!(player.current_index(), 0);
    assert_eq!(player.current_track().unwrap().title, "Track 1");

    // Test next track
    player.next();
    assert_eq!(player.current_index(), 1);

    // Test previous track
    player.previous();
    assert_eq!(player.current_index(), 0);

    // Test repeat modes
    player.set_repeat_mode(RepeatMode::All);
    assert_eq!(player.repeat_mode(), RepeatMode::All);
    player.set_repeat_mode(RepeatMode::One);
    assert_eq!(player.repeat_mode(), RepeatMode::One);

    // Test volume and balance
    player.set_volume(0.65);
    player.set_balance(-0.5);

    // Test clear queue
    player.clear_queue();
    assert_eq!(player.queue_tracks().len(), 0);
    println!("[E2E TEST 4] PASSED: Player state machine and queue verified.");
}

#[tokio::test]
async fn test_e2e_05_library_and_persistence() {
    println!("\n=== [E2E TEST 5] Library, Playlists & Persistence ===");
    let paths = AppPaths::new(AppEdition::Full);
    let temp_lib_file = std::env::temp_dir().join(format!("drxwn_e2e_lib_{}.json", std::process::id()));
    let temp_set_file = std::env::temp_dir().join(format!("drxwn_e2e_set_{}.json", std::process::id()));

    let mut custom_paths = paths.clone();
    custom_paths.library_file = temp_lib_file.clone();
    custom_paths.settings_file = temp_set_file.clone();

    let lib = LibraryManager::new(custom_paths.clone());
    let t1 = Track::new("t_like_1", "Shadow Moses", "Bring Me The Horizon", 243);
    let t2 = Track::new("t_like_2", "Sleepwalking", "Bring Me The Horizon", 230);

    // Test Liked tracks
    let is_liked = lib.toggle_like(t1.clone()).expect("Toggle like");
    assert!(is_liked, "Track should now be liked");
    assert_eq!(lib.get_liked().len(), 1);
    assert!(lib.is_liked("t_like_1"));

    let is_unliked = lib.toggle_like(t1.clone()).expect("Toggle like again");
    assert!(!is_unliked, "Track should now be unliked");
    assert_eq!(lib.get_liked().len(), 0);

    let _ = lib.toggle_like(t2.clone());
    assert_eq!(lib.get_liked().len(), 1);

    // Test Playlists
    let pl = lib.create_playlist("Metalcore Essentials").expect("Create playlist");
    assert_eq!(pl.name, "Metalcore Essentials");
    assert_eq!(lib.get_playlists().len(), 1);

    lib.add_to_playlist(&pl.id, t1.clone()).expect("Add to playlist");
    let updated_pl = lib.get_playlists().into_iter().find(|p| p.id == pl.id).unwrap();
    assert_eq!(updated_pl.tracks.len(), 1);
    assert_eq!(updated_pl.tracks[0].id, "t_like_1");

    // Test History
    lib.record_history(t1.clone()).expect("Record history");
    assert_eq!(lib.get_history().len(), 1);
    lib.clear_history().expect("Clear history");
    assert_eq!(lib.get_history().len(), 0);

    // Test Settings atomic store
    let mut settings = AppSettings::default();
    settings.volume = 0.92;
    settings.theme = "satanic".to_string();
    Store::save_atomic(&custom_paths.settings_file, &settings).expect("Save settings");

    let loaded_settings: AppSettings = Store::load_or_default(&custom_paths.settings_file);
    assert_eq!(loaded_settings.volume, 0.92);
    assert_eq!(loaded_settings.theme, "satanic");

    let _ = std::fs::remove_file(temp_lib_file);
    let _ = std::fs::remove_file(temp_set_file);
    println!("[E2E TEST 5] PASSED: Library, playlists and persistence verified.");
}

#[tokio::test]
async fn test_e2e_06_lyrics_parsing_and_sync() {
    println!("\n=== [E2E TEST 6] LRCLIB Lyrics Parsing & Time Synchronization ===");
    let lrc = r#"
[00:00.00]Can you feel my heart?
[00:15.50]I can't drown my demons
[00:19.20]They know how to swim
[00:35.00]Outro
"#;
    let lines = LyricsService::parse_lrc(lrc);
    assert_eq!(lines.len(), 4);
    assert_eq!(lines[0].text, "Can you feel my heart?");
    assert_eq!(lines[0].time_ms, 0);
    assert_eq!(lines[1].text, "I can't drown my demons");
    assert_eq!(lines[1].time_ms, 15500);
    assert_eq!(lines[2].text, "They know how to swim");
    assert_eq!(lines[2].time_ms, 19200);

    // Check active line search algorithm
    let cur_ms = 17000; // between 15.5s and 19.2s
    let mut active_idx = 0;
    for (i, line) in lines.iter().enumerate() {
        if cur_ms >= line.time_ms {
            active_idx = i;
        }
    }
    assert_eq!(active_idx, 1, "Active lyric line must be 'I can't drown my demons'");
    println!("[E2E TEST 6] PASSED: LRCLIB parser and active lyric sync verified.");
}

#[tokio::test]
async fn test_e2e_07_nostr_identity_and_chathub() {
    println!("\n=== [E2E TEST 7] Nostr Cryptographic Identity & ChatHub ===");
    let id_a = NostrIdentity::generate();
    let id_b = NostrIdentity::generate();

    assert_eq!(id_a.public_key_hex().len(), 64);
    assert_eq!(id_b.public_key_hex().len(), 64);

    let mut hub = ChatHubManager::new();
    let friend = drxwnify_friends::FriendStatus {
        pubkey: id_b.public_key_hex(),
        name: "fend1x".to_string(),
        is_online: true,
        current_track: Some(Track::new("bmth:1", "Can You Feel My Heart", "Bring Me The Horizon", 227)),
        position_ms: Some(45000),
    };
    hub.add_friend(friend);
    let friends = hub.get_friends();
    assert_eq!(friends.len(), 1, "ChatHub should contain added friend");
    assert_eq!(friends[0].name, "fend1x");

    let msg = ChatMessage {
        id: "msg_test_1".to_string(),
        from_pubkey: id_a.public_key_hex(),
        to_pubkey: id_b.public_key_hex(),
        text: "Listening to Bring Me The Horizon!".to_string(),
        timestamp: chrono::Utc::now().timestamp_millis(),
        track_attachment: Some(Track::new("bmth:1", "Can You Feel My Heart", "Bring Me The Horizon", 227)),
    };

    hub.add_message(msg.clone());
    let conversation = hub.get_messages(&id_b.public_key_hex());
    assert_eq!(conversation.len(), 1);
    assert_eq!(conversation[0].text, "Listening to Bring Me The Horizon!");
    assert!(conversation[0].track_attachment.is_some());
    println!("[E2E TEST 7] PASSED: Nostr identity and ChatHub verified.");
}
