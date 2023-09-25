use std::{io, net::SocketAddr, sync::Arc};

use anyhow::bail;
use tokio::{
    io::AsyncWriteExt,
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpStream,
    },
};

use crate::io::{
    codec::MinecraftCodec,
    packet::server::login::{Disconnect, ServerLoginPacket},
    Readable, Writable,
};

use self::handshake::ConnectionResult;

use super::{NewPlayer, ServerState};

mod handshake;

/// A worker for a single client.
pub struct Worker {
    addr: SocketAddr,
    server_state: Arc<ServerState>,
    new_player_sender: flume::Sender<NewPlayer>,
    reader: Reader,
    writer: Writer,
}

impl Worker {
    pub fn new(
        stream: TcpStream,
        addr: SocketAddr,
        server_state: Arc<ServerState>,
        new_player_sender: flume::Sender<NewPlayer>,
    ) -> Self {
        let (reader, writer) = split_stream(stream);
        Self {
            addr,
            server_state,
            reader,
            writer,
            new_player_sender,
        }
    }

    /// Runs this worker.
    pub async fn run(mut self) -> anyhow::Result<()> {
        match handshake::perform_handshake(&mut self).await {
            Ok(v) => match v {
                ConnectionResult::Status => {
                    // status, do nothing
                }
                ConnectionResult::Login(profile) => {
                    let (send1, recv1) = flume::unbounded();
                    let (send2, recv2) = flume::unbounded();

                    let profile = Arc::new(profile);
                    let new_player = NewPlayer {
                        profile: profile.clone(),
                        sender: send1,
                        receiver: recv2,
                    };

                    self.new_player_sender.send_async(new_player).await?;

                    tokio::select! {
                        x = self.reader.run(send2) => {
                            let err = x.unwrap_err();
                            log::info!("{:?} disconnected: {:?}", profile.name, err);
                        },
                        x = self.writer.run(recv1) => {
                            let err = x.unwrap_err();
                            log::info!("{:?} disconnected: {:?}", profile.name, err);
                        }
                    }
                }
            },
            Err(e) => {
                log::error!("Error: {:?}", e);
                self.writer
                    .write(ServerLoginPacket::Disconnect(Disconnect {
                        data: format!(r#"{{"text": "{e:?}"}}"#),
                    }))
                    .await?;
            }
        }
        Ok(())
    }
}

/// Split a stream.
fn split_stream(t: TcpStream) -> (Reader, Writer) {
    let (reader, writer) = t.into_split();
    (
        Reader {
            reader,
            codec: MinecraftCodec::new(),
            buf: [0; 512],
        },
        Writer {
            writer,
            codec: MinecraftCodec::new(),
            writing_buf: Vec::with_capacity(512),
        },
    )
}

/// A reader half of a connection.
pub struct Reader {
    reader: OwnedReadHalf,
    codec: MinecraftCodec,
    buf: [u8; 512],
}

impl Reader {
    /// Run this reader.
    pub async fn run<P: Readable + Send + Sync + 'static>(
        mut self,
        sender: flume::Sender<P>,
    ) -> anyhow::Result<!> {
        loop {
            let v = self.read().await?;
            sender.send_async(v).await?;
        }
    }

    /// Read a packet from this reader.
    pub async fn read<P: Readable>(&mut self) -> anyhow::Result<P> {
        loop {
            self.reader.readable().await?;
            let read = match self.reader.try_read(&mut self.buf) {
                Ok(0) => bail!("EOF"),
                Ok(v) => v,
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => continue,
                Err(e) => return Err(e.into()),
            };
            self.codec.accept_data(&self.buf[..read]);
            if let Some(packet) = self.codec.read_packet()? {
                return Ok(packet);
            }
        }
    }
}

/// A reader half of a connection.
pub struct Writer {
    writer: OwnedWriteHalf,
    codec: MinecraftCodec,
    writing_buf: Vec<u8>,
}

impl Writer {
    /// Run this reader in a separate task.
    pub async fn run<P: Writable + Send + Sync + 'static>(
        mut self,
        receiver: flume::Receiver<P>,
    ) -> anyhow::Result<!> {
        loop {
            let v = receiver.recv_async().await?;
            self.write(v).await?;
        }
    }

    /// Write a packet to this writer.
    pub async fn write<P: Writable>(&mut self, value: P) -> anyhow::Result<()> {
        self.codec.write_packet(value, &mut self.writing_buf)?;
        //log::debug!("Writing {:?}", self.writing_buf);
        self.writer.write_all(&self.writing_buf).await?;
        self.writing_buf.truncate(0);
        Ok(())
    }
}
