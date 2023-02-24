use num::BigInt;
use reqwest::StatusCode;
use rsa::RsaPublicKey;
use rsa::pkcs8::EncodePublicKey;
use serde::Deserialize;
use serde::Serialize;
use sha1::Sha1;
use sha1::Digest;
use thiserror::Error;
use uuid::Uuid;

/// Yggdrasil authentication URL.
const AUTH_URL: &str = "https://sessionserver.mojang.com/session/minecraft/hasJoined";

/// Authentication helper.
pub struct MinecraftAuthenticator {
    
    /// The player's username.
    username: String,
    /// The hash, computed from values
    /// used in the connection.
    server_hash: String,
    /// The player's IP address.
    ip: Option<String>
}
#[derive(Debug, Error)]
pub enum MCAuthError {
    #[error("RSA key error: {0}")]
    SPKIError(rsa::pkcs8::spki::Error),
    #[error("Connection error: {0}")]
    ReqwestError(reqwest::Error),
    #[error("JSON error: {0}")]
    JSONError(serde_json::Error),
    #[error("Authentication failure: {0}")]
    AuthFailure(String)
}
pub type MCAuthResult<T> = std::result::Result<T, MCAuthError>;

impl MinecraftAuthenticator {
    /// Create a new authenticator.
    pub fn new(username: String, ip: Option<String>, server_id: String, shared_secret: [u8; 16], server_public_key: &RsaPublicKey) -> MCAuthResult<Self> {
        let mut id_hasher = Sha1::default();
        id_hasher.update(server_id.as_bytes());
        id_hasher.update(shared_secret);
        id_hasher.update(server_public_key.to_public_key_der().map_err(MCAuthError::SPKIError)?.as_bytes());
        let server_hash = minecraft_digest(id_hasher);
        Ok(Self {
            username,
            server_hash,
            ip
        })
    }

    /// Attempts to authenticate this client.
    pub async fn authenticate(self) -> MCAuthResult<Profile> {
        let cl = reqwest::Client::new();
        let mut req = cl.get(AUTH_URL).query(&[("username", self.username), ("serverId", self.server_hash)]);
        if let Some(ip) = self.ip {
            req = req.query(&("ip", ip));
        }
        let received = req.send().await.map_err(MCAuthError::ReqwestError)?;
        match received.error_for_status_ref() {
            Ok(_) => {
                let text = received.text().await.map_err(MCAuthError::ReqwestError)?;
                serde_json::from_str(&text).map_err(MCAuthError::JSONError)
            },
            Err(code) => {
                if code.status() == Some(StatusCode::FORBIDDEN) {
                    Err(MCAuthError::AuthFailure(received.text().await.map_err(MCAuthError::ReqwestError)?))
                } else {
                    Err(MCAuthError::ReqwestError(code))
                }
            },
        }
    }
}


/// The profile representing this user.
#[derive(Debug, Serialize, Deserialize)]
pub struct Profile {
    /// The player's UUID.
    pub id: Uuid,
    /// The player's username.
    pub name: String,
    /// The player's properties.
    pub properties: Vec<ProfileProperty>
}

/// A profile property.
#[derive(Debug, Serialize, Deserialize)]
pub struct ProfileProperty {
    /// The name of this property.
    pub name: String,
    /// The value of this property.
    pub value: String,
    /// The signature of this property. Signed by
    /// the Yggdrasil public key.
    pub signature: String
}



/// Helper function to generate a Minecraft 
/// compatible non-standard SHA-1 digest. 
fn minecraft_digest(sha1: Sha1) -> String {
    let data = sha1.finalize();
    let bigint = BigInt::from_signed_bytes_be(&data);
    bigint.to_str_radix(16)
}