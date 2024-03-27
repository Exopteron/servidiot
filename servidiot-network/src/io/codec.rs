use std::io::Cursor;

pub use self::cryptor::Cryptor;

use super::{Writable, VarInt, Readable};




/// A Minecraft codec.
pub struct MinecraftCodec {
    cryptor: Option<Cryptor>,
    received_buf: Vec<u8>,
    staging_buf: Vec<u8>
}
impl Default for MinecraftCodec {
    fn default() -> Self {
        Self::new()
    }
}
impl MinecraftCodec {
    pub fn new() -> Self {
        Self {
            cryptor: None,
            received_buf: Vec::with_capacity(512),
            staging_buf: Vec::with_capacity(512)
        }
    }

    /// Enable encryption on this codec.
    pub fn enable_encryption(&mut self, cryptor: Cryptor) {
        self.cryptor = Some(cryptor);
    }

    /// Encode a packet to `target`.
    pub fn write_packet<P: Writable>(&mut self, packet: P, target: &mut Vec<u8>) -> anyhow::Result<()> {
        packet.write_to(&mut self.staging_buf)?;
        VarInt::try_from(self.staging_buf.len())?.write_to(target)?;
        target.append(&mut self.staging_buf);
        if let Some(cryptor) = &mut self.cryptor {
            cryptor.encrypt(target);
        }
        self.staging_buf.truncate(0);
        Ok(())
    }

    /// Accept some data in. Returns a view of 
    /// the processed data.
    pub fn accept_data(&mut self, data: &[u8]) -> &[u8] {
        let len = self.received_buf.len();
        let new_len = len + data.len();
        self.received_buf.extend_from_slice(data);
        if let Some(c) = &mut self.cryptor {
            c.decrypt(&mut self.received_buf[len..new_len]);
        }
        &self.received_buf[len..new_len]
    }

    /// Try to read a packet. Returns `None` if
    /// there are not enough bytes to read a 
    /// full packet.
    pub fn read_packet<P: Readable>(&mut self) -> anyhow::Result<Option<P>> {
        let mut cursor = Cursor::new(self.received_buf.as_slice());
        if let Ok(v) = VarInt::read_from(&mut cursor) {
            let packet_length: usize = v.0.try_into()?;
            if cursor.remaining_slice().len() >= packet_length {
                // we have enough data

                let varint_length = self.received_buf.len() - cursor.remaining_slice().len();

                // read the packet
                let packet = P::read_from(&mut cursor)?;


                // shrink the received buffer
                let end_of_packet = varint_length + packet_length;
                let new_len = self.received_buf.len().saturating_sub(end_of_packet);
                if end_of_packet <= self.received_buf.len() {
                    self.received_buf.copy_within(end_of_packet.., 0);
                    self.received_buf.truncate(new_len);
                } else {
                    self.received_buf.truncate(0);
                }

                // return the packet
                return Ok(Some(packet));
            } else {
                // not enough data
                return Ok(None);
            }
        }
        Ok(None)
    }
}



mod cryptor {
    use aes::cipher::{
        generic_array::GenericArray, BlockDecryptMut, BlockEncryptMut, KeyIvInit,
    };
    
    /// Minecraft AES-CBC cryptor.

    type Aes128Cfb8Enc = cfb8::Encryptor<aes::Aes128>;
    type Aes128Cfb8Dec = cfb8::Decryptor<aes::Aes128>;

    /// A minecraft cryptor.
    pub struct Cryptor {
        encryptor: Aes128Cfb8Enc,
        decryptor: Aes128Cfb8Dec,
    }

    impl Cryptor {
        /// Initializes this cryptor.
        pub fn init(key: [u8; 16]) -> Self {
            Self {
                encryptor: Aes128Cfb8Enc::new(&key.into(), &key.into()),
                decryptor: Aes128Cfb8Dec::new(&key.into(), &key.into()),
            }
        }

        /// Decrypt the buffer data in-place.
        pub fn decrypt(&mut self, target: &mut [u8]) {
            // hacky workaround
            let x = unsafe {
                std::mem::transmute::<&mut [u8], &mut [GenericArray<u8, aes::cipher::consts::U1>]>(
                    target,
                )
            };
            assert_eq!(x.len(), target.len());
            self.decryptor.decrypt_blocks_mut(x);
        }

        /// Encrypt the buffer data in-place.
        pub fn encrypt(&mut self, target: &mut [u8]) {
            // hacky workaround
            let x = unsafe {
                std::mem::transmute::<&mut [u8], &mut [GenericArray<u8, aes::cipher::consts::U1>]>(
                    target,
                )
            };
            assert_eq!(x.len(), target.len());
            self.encryptor.encrypt_blocks_mut(x);
        }
    }
}
