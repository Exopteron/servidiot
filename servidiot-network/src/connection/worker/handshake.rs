use anyhow::bail;
use rand::Rng;
use rsa::{Pkcs1v15Encrypt, pkcs8::EncodePublicKey};
use servidiot_yggdrasil::authenticate::{MinecraftAuthenticator, Profile};

use crate::{
    io::{
        packet::{
            client::{
                handshake::{ClientHandshakePacket, NextState},
                login::ClientLoginPacket,
            },
            server::login::{EncryptionRequest, ServerLoginPacket},
        },
        LengthPrefixedVec,
    },
};

use super::Worker;

pub enum ConnectionResult {
    Login(Profile),
    Status,
}

pub async fn handle_status(_worker: &mut Worker) -> anyhow::Result<ConnectionResult> {
    log::error!("todo status");
    Ok(ConnectionResult::Status)
}

/// Do the handshake.
pub async fn perform_handshake(worker: &mut Worker) -> anyhow::Result<ConnectionResult> {
    let ClientHandshakePacket::Handshake(handshake) =
        worker.reader.read::<ClientHandshakePacket>().await?;
    if handshake.next_state == NextState::Status {
        return handle_status(worker).await;
    }

    if handshake.protocol_version.0 != 5 {
        bail!("wrong protocol version {:?}", handshake.protocol_version);
    }

    let ClientLoginPacket::LoginStart(login_start) = worker.reader.read::<ClientLoginPacket>().await? else {
        bail!("unexpected packet in login sequence");
    };

    let player_name = login_start.name;

    let verify_token = rand::thread_rng().gen::<[u8; 4]>();

    worker
        .writer
        .write(ServerLoginPacket::EncryptionRequest(EncryptionRequest {
            server_id: String::new(),
            public_key: LengthPrefixedVec::new(
                worker
                    .server_state
                    .rsa_key
                    .to_public_key()
                    .to_public_key_der()?
                    .to_vec(),
            ),
            verify_token: LengthPrefixedVec::new(verify_token.to_vec()),
        }))
        .await?;

    let ClientLoginPacket::EncryptionResponse(encryption_response) = worker.reader.read::<ClientLoginPacket>().await? else {
            bail!("unexpected packet in login sequence");
    };
    let encrypted_secret = encryption_response.shared_secret.0;
    let encrypted_token = encryption_response.verify_token.0;

    let shared_secret_vec = worker
        .server_state
        .rsa_key
        .decrypt(Pkcs1v15Encrypt, &encrypted_secret)?;
    let decrypted_token = worker
        .server_state
        .rsa_key
        .decrypt(Pkcs1v15Encrypt, &encrypted_token)?;
    if decrypted_token != verify_token {
        bail!("invalid crypto verification token");
    }

    if shared_secret_vec.len() != 16 {
        bail!("bad shared secret length");
    }

    let mut shared_secret = [0; 16];
    shared_secret.copy_from_slice(&shared_secret_vec);

    let authenticator = MinecraftAuthenticator::new(
        player_name.clone(),
        None,
        String::new(),
        shared_secret,
        &worker.server_state.rsa_key.to_public_key(),
    )?;

    let profile = authenticator.authenticate().await?;

    if profile.name != player_name {
        bail!("name mismatch")
    }


    Ok(ConnectionResult::Login(profile))
}
