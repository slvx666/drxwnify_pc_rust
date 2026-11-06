use drxwnify_shared_models::Track;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: String,
    pub from_pubkey: String,
    pub to_pubkey: String,
    pub text: String,
    pub timestamp: i64,
    pub track_attachment: Option<Track>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FriendStatus {
    pub pubkey: String,
    pub name: String,
    pub is_online: bool,
    pub current_track: Option<Track>,
    pub position_ms: Option<i64>,
}

pub struct ChatHubManager {
    friends: Vec<FriendStatus>,
    messages: Vec<ChatMessage>,
}

impl ChatHubManager {
    pub fn new() -> Self {
        let friends = vec![
            FriendStatus {
                pubkey: "npub1slvx666".to_string(),
                name: "slvx666 (drxwnslvt)".to_string(),
                is_online: true,
                current_track: Some(Track {
                    id: "drown_bmth".to_string(),
                    title: "Drown".to_string(),
                    artists: vec![drxwnify_shared_models::ArtistRef {
                        id: Some("bmth".to_string()),
                        name: "Bring Me The Horizon".to_string(),
                    }],
                    album: None,
                    duration_sec: 222,
                    cover: None,
                    explicit: false,
                    youtube_id: Some("drown_bmth".to_string()),
                    isrc: None,
                    local_path: None,
                    bitrate_kbps: Some(320),
                    source: Some("InnerTube".to_string()),
                }),
                position_ms: Some(45000),
            },
            FriendStatus {
                pubkey: "npub1fend1x".to_string(),
                name: "fend1x".to_string(),
                is_online: true,
                current_track: Some(Track {
                    id: "res_home".to_string(),
                    title: "Resonance".to_string(),
                    artists: vec![drxwnify_shared_models::ArtistRef {
                        id: Some("home".to_string()),
                        name: "HOME".to_string(),
                    }],
                    album: None,
                    duration_sec: 212,
                    cover: None,
                    explicit: false,
                    youtube_id: Some("res_home".to_string()),
                    isrc: None,
                    local_path: None,
                    bitrate_kbps: Some(320),
                    source: Some("InnerTube".to_string()),
                }),
                position_ms: Some(89000),
            },
            FriendStatus {
                pubkey: "npub1clavdiy".to_string(),
                name: "clavdiy".to_string(),
                is_online: false,
                current_track: None,
                position_ms: None,
            },
        ];
        Self {
            friends,
            messages: Vec::new(),
        }
    }

    pub fn get_friends(&self) -> &[FriendStatus] {
        &self.friends
    }

    pub fn add_friend(&mut self, friend: FriendStatus) {
        if let Some(existing) = self.friends.iter_mut().find(|f| f.pubkey == friend.pubkey) {
            *existing = friend;
        } else {
            self.friends.push(friend);
        }
    }

    pub fn add_message(&mut self, msg: ChatMessage) {
        self.messages.push(msg);
    }

    pub fn get_messages(&self, friend_pubkey: &str) -> Vec<ChatMessage> {
        self.messages
            .iter()
            .filter(|m| m.from_pubkey == friend_pubkey || m.to_pubkey == friend_pubkey)
            .cloned()
            .collect()
    }
}

impl Default for ChatHubManager {
    fn default() -> Self {
        Self::new()
    }
}
