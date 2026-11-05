#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use drxwnify_audio_engine::AudioEngine;
use drxwnify_core::{AppEdition, AppPaths, EventBus};
use drxwnify_friends::{ChatMessage, FriendStatus, ChatHubManager};
use drxwnify_library::LibraryManager;
use drxwnify_lyrics::LyricsService;
use drxwnify_network::HttpClient;
use drxwnify_persistence::Store;
use drxwnify_playback::Player;
use drxwnify_search::SearchService;
use drxwnify_shared_models::*;
use parking_lot::Mutex;
use std::sync::Arc;
use tauri::{State, Window};

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct PlaybackStateDto {
    is_playing: bool,
    position_ms: i64,
    current_track: Option<Track>,
    queue: Vec<Track>,
    current_index: usize,
    repeat_mode: RepeatMode,
}

struct AppState {
    player: Player,
    library: Arc<LibraryManager>,
    search_service: SearchService,
    lyrics_service: LyricsService,
    chathub: Arc<Mutex<ChatHubManager>>,
    paths: AppPaths,
    settings: Arc<Mutex<AppSettings>>,
    accounts: Arc<Mutex<Accounts>>,
}

#[tauri::command]
fn play_track(track: Track, state: State<'_, AppState>) -> Result<(), String> {
    state.player.play_track(track);
    Ok(())
}

#[tauri::command]
fn play_tracks(tracks: Vec<Track>, start_index: usize, state: State<'_, AppState>) -> Result<(), String> {
    state.player.play_tracks(tracks, start_index);
    Ok(())
}

#[tauri::command]
fn play_queue_index(index: usize, state: State<'_, AppState>) -> Result<(), String> {
    state.player.play_queue_index(index);
    Ok(())
}

#[tauri::command]
fn clear_queue(state: State<'_, AppState>) -> Result<(), String> {
    state.player.clear_queue();
    Ok(())
}

#[tauri::command]
fn toggle_play_pause(state: State<'_, AppState>) -> Result<(), String> {
    state.player.toggle_play_pause();
    Ok(())
}

#[tauri::command]
fn seek(position_ms: i64, state: State<'_, AppState>) -> Result<(), String> {
    state.player.seek(position_ms);
    Ok(())
}

#[tauri::command]
fn set_volume(volume: f32, state: State<'_, AppState>) -> Result<(), String> {
    state.player.set_volume(volume);
    let mut s = state.settings.lock();
    s.volume = volume;
    let _ = Store::save_atomic(&state.paths.settings_file, &*s);
    Ok(())
}

#[tauri::command]
fn set_balance(balance: f32, state: State<'_, AppState>) -> Result<(), String> {
    state.player.set_balance(balance);
    let mut s = state.settings.lock();
    s.balance = balance;
    let _ = Store::save_atomic(&state.paths.settings_file, &*s);
    Ok(())
}

#[tauri::command]
fn next_track(state: State<'_, AppState>) -> Result<(), String> {
    state.player.next();
    Ok(())
}

#[tauri::command]
fn previous_track(state: State<'_, AppState>) -> Result<(), String> {
    state.player.previous();
    Ok(())
}

#[tauri::command]
fn shuffle(state: State<'_, AppState>) -> Result<(), String> {
    state.player.shuffle();
    Ok(())
}

#[tauri::command]
fn set_repeat_mode(mode: String, state: State<'_, AppState>) -> Result<(), String> {
    let rep = match mode.to_lowercase().as_str() {
        "one" => RepeatMode::One,
        "all" => RepeatMode::All,
        _ => RepeatMode::Off,
    };
    state.player.set_repeat_mode(rep);
    Ok(())
}

#[tauri::command]
fn get_playback_state(state: State<'_, AppState>) -> PlaybackStateDto {
    PlaybackStateDto {
        is_playing: state.player.is_playing(),
        position_ms: state.player.get_position_ms(),
        current_track: state.player.current_track(),
        queue: state.player.queue_tracks(),
        current_index: state.player.current_index(),
        repeat_mode: state.player.repeat_mode(),
    }
}

#[tauri::command]
fn get_spectrum(state: State<'_, AppState>) -> Vec<f32> {
    state.player.get_spectrum()
}

#[tauri::command]
fn get_liked(state: State<'_, AppState>) -> Vec<Track> {
    state.library.get_liked()
}

#[tauri::command]
fn toggle_like(track: Track, state: State<'_, AppState>) -> Result<bool, String> {
    state.library.toggle_like(track).map_err(|e| e.to_string())
}

#[tauri::command]
async fn search(query: String, state: State<'_, AppState>) -> Result<drxwnify_search::SearchResults, String> {
    state.search_service.search(&query).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_lyrics(track: Track, state: State<'_, AppState>) -> Result<LyricsData, String> {
    state.lyrics_service.fetch_lyrics(&track).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn translate_lyrics(lines: Vec<String>, target_lang: String, state: State<'_, AppState>) -> Result<Vec<String>, String> {
    state.lyrics_service.translate_batch(&lines, &target_lang).await.map_err(|e| e.to_string())
}

#[tauri::command]
fn get_history(state: State<'_, AppState>) -> Vec<Played> {
    state.library.get_history()
}

#[tauri::command]
fn clear_history(state: State<'_, AppState>) -> Result<(), String> {
    state.library.clear_history().map_err(|e| e.to_string())
}

#[tauri::command]
fn get_playlists(state: State<'_, AppState>) -> Vec<Playlist> {
    state.library.get_playlists()
}

#[tauri::command]
fn create_playlist(name: String, state: State<'_, AppState>) -> Result<Playlist, String> {
    state.library.create_playlist(&name).map_err(|e| e.to_string())
}

#[tauri::command]
fn add_to_playlist(playlist_id: String, track: Track, state: State<'_, AppState>) -> Result<(), String> {
    state.library.add_to_playlist(&playlist_id, track).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_settings(state: State<'_, AppState>) -> AppSettings {
    state.settings.lock().clone()
}

#[tauri::command]
fn save_settings(settings: AppSettings, state: State<'_, AppState>) -> Result<(), String> {
    *state.settings.lock() = settings.clone();
    Store::save_atomic(&state.paths.settings_file, &settings).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_accounts(state: State<'_, AppState>) -> Accounts {
    state.accounts.lock().clone()
}

#[tauri::command]
fn save_accounts(accounts: Accounts, state: State<'_, AppState>) -> Result<(), String> {
    *state.accounts.lock() = accounts.clone();
    Store::save_atomic(&state.paths.accounts_file, &accounts).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_friends(state: State<'_, AppState>) -> Vec<FriendStatus> {
    state.chathub.lock().get_friends().to_vec()
}

#[tauri::command]
fn get_messages(friend_pubkey: String, state: State<'_, AppState>) -> Vec<ChatMessage> {
    state.chathub.lock().get_messages(&friend_pubkey)
}

#[tauri::command]
fn send_message(to_pubkey: String, text: String, track: Option<Track>, state: State<'_, AppState>) -> Result<ChatMessage, String> {
    let msg = ChatMessage {
        id: format!("msg_{}", chrono::Utc::now().timestamp_millis()),
        from_pubkey: "me".to_string(),
        to_pubkey,
        text,
        timestamp: chrono::Utc::now().timestamp_millis(),
        track_attachment: track,
    };
    state.chathub.lock().add_message(msg.clone());
    Ok(msg)
}

#[tauri::command]
fn import_backup(zip_path: String, state: State<'_, AppState>) -> Result<usize, String> {
    let imported = drxwnify_persistence::PhoneBackupImporter::import_backup_zip(std::path::Path::new(&zip_path))
        .map_err(|e| e.to_string())?;
    let count = imported.liked.len();
    for t in imported.liked {
        let _ = state.library.toggle_like(t);
    }
    Ok(count)
}

#[tauri::command]
fn minimize_window(window: Window) -> Result<(), String> {
    window.minimize().map_err(|e| e.to_string())
}

#[tauri::command]
fn maximize_window(window: Window) -> Result<(), String> {
    if window.is_maximized().unwrap_or(false) {
        window.unmaximize().map_err(|e| e.to_string())
    } else {
        window.maximize().map_err(|e| e.to_string())
    }
}

#[tauri::command]
fn close_window(window: Window) -> Result<(), String> {
    window.close().map_err(|e| e.to_string())
}

fn main() {
    let paths = AppPaths::new(AppEdition::Full);
    let _ = paths.ensure_directories();

    let event_bus = EventBus::default();
    let audio_engine = Arc::new(AudioEngine::new(event_bus.clone(), None));
    let http = HttpClient::default();
    let player = Player::new(audio_engine, event_bus.clone(), http.clone());
    let library = Arc::new(LibraryManager::new(paths.clone()));

    let search_service = SearchService::new(http.clone());
    let lyrics_service = LyricsService::new(http);
    let chathub = Arc::new(Mutex::new(ChatHubManager::new()));

    let settings: AppSettings = Store::load_or_default(&paths.settings_file);
    let accounts: Accounts = Store::load_or_default(&paths.accounts_file);

    let app_state = AppState {
        player,
        library,
        search_service,
        lyrics_service,
        chathub,
        settings: Arc::new(Mutex::new(settings)),
        accounts: Arc::new(Mutex::new(accounts)),
        paths,
    };

    tauri::Builder::default()
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            play_track,
            play_tracks,
            play_queue_index,
            clear_queue,
            toggle_play_pause,
            seek,
            set_volume,
            set_balance,
            next_track,
            previous_track,
            shuffle,
            set_repeat_mode,
            get_playback_state,
            get_spectrum,
            get_liked,
            toggle_like,
            search,
            get_lyrics,
            translate_lyrics,
            get_history,
            clear_history,
            get_playlists,
            create_playlist,
            add_to_playlist,
            get_settings,
            save_settings,
            get_accounts,
            save_accounts,
            get_friends,
            get_messages,
            send_message,
            import_backup,
            minimize_window,
            maximize_window,
            close_window,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Drxwnify Full");
}
