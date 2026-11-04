use drxwnify_audio_engine::dsp::DspProcessor;
use drxwnify_audio_engine::fft::SpectrumAnalyzer;
use drxwnify_core::paths::{AppEdition, AppPaths};
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
