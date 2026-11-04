use drxwnify_core::{DrxwnifyError, DrxwnifyResult};
use secp256k1::hashes::sha256;
use secp256k1::rand::rngs::OsRng;
use secp256k1::schnorr::Signature;
use secp256k1::{Keypair, Message, Secp256k1, XOnlyPublicKey};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NostrEvent {
    pub id: String,
    pub pubkey: String,
    pub created_at: i64,
    pub kind: u32,
    pub tags: Vec<Vec<String>>,
    pub content: String,
    pub sig: String,
}

pub struct NostrIdentity {
    secp: Secp256k1<secp256k1::All>,
    keypair: Keypair,
}

impl NostrIdentity {
    pub fn generate() -> Self {
        let secp = Secp256k1::new();
        let mut rng = OsRng;
        let keypair = Keypair::new(&secp, &mut rng);
        Self { secp, keypair }
    }

    pub fn from_hex_secret(hex_secret: &str) -> DrxwnifyResult<Self> {
        let secp = Secp256k1::new();
        let bytes = hex::decode(hex_secret)
            .map_err(|e| DrxwnifyError::Nostr(format!("Invalid hex secret: {e}")))?;
        let keypair = Keypair::from_seckey_slice(&secp, &bytes)
            .map_err(|e| DrxwnifyError::Nostr(format!("Invalid secret key: {e}")))?;
        Ok(Self { secp, keypair })
    }

    pub fn public_key_hex(&self) -> String {
        let (x_only, _) = self.keypair.x_only_public_key();
        x_only.to_string()
    }

    pub fn secret_key_hex(&self) -> String {
        hex::encode(self.keypair.secret_key().secret_bytes())
    }

    pub fn sign_event(&self, kind: u32, tags: Vec<Vec<String>>, content: String) -> NostrEvent {
        let created_at = chrono::Utc::now().timestamp();
        let pubkey = self.public_key_hex();

        let serialized = serde_json::json!([
            0,
            pubkey,
            created_at,
            kind,
            tags,
            content
        ]).to_string();

        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(serialized.as_bytes());
        let hash = hasher.finalize();
        let id_hex = hex::encode(hash);

        let msg = Message::from_digest_slice(&hash).expect("32 bytes hash");
        let sig = self.secp.sign_schnorr_no_aux_rand(&msg, &self.keypair);

        NostrEvent {
            id: id_hex,
            pubkey,
            created_at,
            kind,
            tags,
            content,
            sig: sig.to_string(),
        }
    }
}
