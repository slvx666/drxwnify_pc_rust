use drxwnify_audio_engine::dsp::DspProcessor;
use drxwnify_audio_engine::fft::SpectrumAnalyzer;
use drxwnify_core::paths::{AppEdition, AppPaths};
use drxwnify_friends::ChatMessage;
use drxwnify_lyrics::LyricsService;
use drxwnify_nostr::NostrIdentity;
use drxwnify_persistence::Store;
use drxwnify_playback::queue::TrackQueue;
use drxwnify_shared_models::*;

#[test]
fn test_dsp_perceptual_volume_scaling() {
    let mut dsp = DspProcessor::new();
    dsp.volume = 0.5;
    dsp.normalize_enabled = false;

    // At 50% volume with x^1.7 curve, gain = 0.5^1.7 = ~0.3077
    let (l, r) = dsp.process_stereo_frame(10000, 10000);
    assert!(l > 2900 && l < 3300, "Left sample scaled: {}", l);
    assert!(r > 2900 && r < 3300, "Right sample scaled: {}", r);
}

#[test]
fn test_dsp_stereo_balance() {
    let mut dsp = DspProcessor::new();
    dsp.volume = 1.0;
    dsp.balance = -1.0; // 100% Left, 0% Right
    dsp.normalize_enabled = false;

    let (l, r) = dsp.process_stereo_frame(10000, 10000);
    assert_eq!(l, 10000);
    assert_eq!(r, 0);

    dsp.balance = 1.0; // 0% Left, 100% Right
    let (l, r) = dsp.process_stereo_frame(10000, 10000);
    assert_eq!(l, 0);
    assert_eq!(r, 10000);
}

#[test]
fn test_track_queue_shuffle_preserves_first_track() {
    let mut q = TrackQueue::new();
    let mut tracks = Vec::new();
    for i in 1..=20 {
        tracks.push(Track::new(format!("t{}", i), format!("Title {}", i), "Artist", 200));
    }

    q.set_tracks(tracks, 0);
    let original_first_id = q.current_track().unwrap().id.clone();

    q.shuffle();

    // Original first track MUST remain at index 0 after shuffle
    assert_eq!(q.current_index(), 0);
    assert_eq!(q.current_track().unwrap().id, original_first_id);
    assert_eq!(q.len(), 20);
}

#[test]
fn test_app_paths_isolation() {
    let full_paths = AppPaths::new(AppEdition::Full);
    let mini_paths = AppPaths::new(AppEdition::Mini);

    assert!(full_paths.root_dir.to_str().unwrap().contains("DrxwnifyRust"));
    assert!(mini_paths.root_dir.to_str().unwrap().contains("DrxwnifyMini"));
    assert_ne!(full_paths.root_dir, mini_paths.root_dir);
}

#[test]
fn test_spectrum_analyzer_32_bands() {
    let mut analyzer = SpectrumAnalyzer::new();
    let bands = analyzer.compute_spectrum();
    assert_eq!(bands.len(), 32);
    for b in bands {
        assert!(b >= 0.0 && b <= 1.0);
    }
}

#[test]
fn test_lyrics_lrc_parsing() {
    let sample_lrc = r#"
[00:12.34]First line of song
[01:05.67]Chorus begins here
[02:30.00]Final outro verse
"#;
    let lines = LyricsService::parse_lrc(sample_lrc);
    assert_eq!(lines.len(), 3);
    assert_eq!(lines[0].text, "First line of song");
    assert_eq!(lines[0].time_ms, 12340);
    assert_eq!(lines[1].text, "Chorus begins here");
    assert_eq!(lines[1].time_ms, 65670);
    assert_eq!(lines[2].text, "Final outro verse");
    assert_eq!(lines[2].time_ms, 150000);
}

#[test]
fn test_nostr_identity_keys_and_signatures() {
    let identity = NostrIdentity::generate();
    let pub_hex = identity.public_key_hex();
    let sec_hex = identity.secret_key_hex();
    assert_eq!(pub_hex.len(), 64);
    assert_eq!(sec_hex.len(), 64);

    let event = identity.sign_event(1, vec![], "Testing Drxwnify Nostr Core".to_string());
    assert_eq!(event.kind, 1);
    assert_eq!(event.content, "Testing Drxwnify Nostr Core");
    assert_eq!(event.pubkey, pub_hex);
    assert_eq!(event.sig.len(), 128);
}

#[test]
fn test_persistence_atomic_store() {
    let temp_file = std::env::temp_dir().join(format!("drxwn_test_store_{}.json", std::process::id()));
    let test_settings = AppSettings {
        volume: 0.85,
        theme: "gruvbox".to_string(),
        ..Default::default()
    };

    Store::save_atomic(&temp_file, &test_settings).expect("save_atomic failed");
    let loaded: AppSettings = Store::load_or_default(&temp_file);
    assert_eq!(loaded.volume, 0.85);
    assert_eq!(loaded.theme, "gruvbox");

    let _ = std::fs::remove_file(temp_file);
}

#[test]
fn test_chathub_messaging_model() {
    let msg = ChatMessage {
        id: "msg_123".to_string(),
        from_pubkey: "npub1slvx".to_string(),
        to_pubkey: "npub1fend1x".to_string(),
        text: "Check out this song!".to_string(),
        timestamp: 1730000000,
        track_attachment: Some(Track::new("bmth:1", "Can You Feel My Heart", "Bring Me The Horizon", 227)),
    };

    let serialized = serde_json::to_string(&msg).expect("Serialize ChatMessage");
    let deserialized: ChatMessage = serde_json::from_str(&serialized).expect("Deserialize ChatMessage");

    assert_eq!(deserialized.id, "msg_123");
    assert_eq!(deserialized.from_pubkey, "npub1slvx");
    assert!(deserialized.track_attachment.is_some());
    assert_eq!(deserialized.track_attachment.unwrap().title, "Can You Feel My Heart");
}
