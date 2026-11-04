use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Accounts {
    #[serde(default)]
    pub spotify_sp_dc: Option<String>,
    #[serde(default)]
    pub spotify_sp_key: Option<String>,
    #[serde(default)]
    pub spotify_access_token: Option<String>,
    #[serde(default)]
    pub spotify_token_expiry: i64,
    #[serde(default)]
    pub spotify_name: Option<String>,
    #[serde(default)]
    pub spotify_user_id: Option<String>,
    #[serde(default)]
    pub vk_token: Option<String>,
    #[serde(default)]
    pub vk_name: Option<String>,
    #[serde(default)]
    pub vk_user_id: Option<String>,
    #[serde(default)]
    pub yandex_token: Option<String>,
    #[serde(default)]
    pub yandex_uid: Option<String>,
    #[serde(default)]
    pub yandex_name: Option<String>,
    #[serde(default)]
    pub lastfm_session: Option<String>,
    #[serde(default)]
    pub lastfm_user: Option<String>,
    #[serde(default = "default_true")]
    pub lastfm_scrobble: bool,
    #[serde(default)]
    pub discord_token: Option<String>,
    #[serde(default)]
    pub soulseek_user: Option<String>,
    #[serde(default)]
    pub soulseek_password: Option<String>,
    #[serde(default = "default_true")]
    pub soulseek_enabled: bool,
    #[serde(default)]
    pub nostr_secret: Option<String>,
}

fn default_true() -> bool { true }
