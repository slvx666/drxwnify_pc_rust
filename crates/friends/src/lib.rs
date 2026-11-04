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
        Self {
            friends: Vec::new(),
            messages: Vec::new(),
        }
    }

    pub fn get_friends(&self) -> &[FriendStatus] {
        &self.friends
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
