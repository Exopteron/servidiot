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
            server::login::{EncryptionRequest, ServerLoginPacket, LoginSuccess},
        },
        LengthPrefixedVec, codec::Cryptor,
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

    log::info!("Read handshake: {:?}", handshake);

    let ClientLoginPacket::LoginStart(login_start) = worker.reader.read::<ClientLoginPacket>().await? else {
        bail!("unexpected packet in login sequence");
    };

    let player_name = login_start.name;

    let verify_token = rand::thread_rng().gen::<[u8; 4]>();


    log::info!("Sending enc request");
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

    log::info!("Got response");
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

    log::info!("Shared secret is {:?}", shared_secret);
    let authenticator = MinecraftAuthenticator::new(
        player_name.clone(),
        None,
        String::new(),
        shared_secret,
        &worker.server_state.rsa_key.to_public_key(),
    )?;

    let profile = authenticator.authenticate().await?;
    log::info!("Authentication succeeded for {:?}@[{:?}]", profile.name, worker.addr);

    if profile.name != player_name {
        bail!("name mismatch")
    }

    log::info!("Enabling encryption");
    // enable enc
    worker.writer.codec.enable_encryption(Cryptor::init(shared_secret));
    worker.reader.codec.enable_encryption(Cryptor::init(shared_secret));

    worker
    .writer
    .write(ServerLoginPacket::LoginSuccess(LoginSuccess {
        username: profile.name.clone(),
        uuid: profile.id.as_hyphenated().to_string()
    }))
    .await?;


    Ok(ConnectionResult::Login(profile))
}
