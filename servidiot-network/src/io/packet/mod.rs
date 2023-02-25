pub mod client;
pub mod server;

macro_rules! def_packets {
    (
        $(
            $packet_name:ident {
                $(
                    $field_name:ident: $field_ty:ty
                ),*
            }
        ),*
    ) => {
        $(
            #[derive(Debug)]
            pub struct $packet_name {
                $(
                    pub $field_name: $field_ty
                ),*
            }

            impl crate::io::Readable for $packet_name {
                fn read_from(data: &mut std::io::Cursor<&[u8]>) -> std::result::Result<Self, anyhow::Error> {
                    Ok(Self {
                        $(
                            $field_name: <$field_ty>::read_from(data)?
                        ),*
                    })
                }
            }
            impl crate::io::Writable for $packet_name {
                fn write_to(&self, target: &mut std::vec::Vec<u8>) -> std::result::Result<(), anyhow::Error> {
                    $(
                        self.$field_name.write_to(target)?;
                    )*
                    Ok(())
                }
            }

        )*
    };
}
pub(crate) use def_packets;

macro_rules! packet_enum {
    (
        $enum_ident:ident {
            $(
                $packet_ident:ident = $id:expr
            ),*
        }
    ) => {
        #[derive(Debug)]
        pub enum $enum_ident {
            $(
                $packet_ident($packet_ident)
            ),*
        }

        impl crate::io::Readable for $enum_ident {
            fn read_from(data: &mut std::io::Cursor<&[u8]>) -> std::result::Result<Self, anyhow::Error> {
                let vint = crate::io::primitives::VarInt::read_from(data)?.0;
                match vint {
                    $(
                        $id => Ok(Self::$packet_ident($packet_ident::read_from(data)?)),
                    )*
                    n => anyhow::bail!("unknown packet ID 0x{n:x} for {:?}!", std::any::type_name::<Self>())
                }
            }
        }
        
        impl crate::io::Writable for $enum_ident {
            fn write_to(&self, target: &mut std::vec::Vec<u8>) -> std::result::Result<(), anyhow::Error> {
                match self {
                    $(
                        Self::$packet_ident(v) => {
                            crate::io::VarInt($id).write_to(target)?;
                            v.write_to(target)?;
                        }
                    ),*
                };
                Ok(())
            }
        }
    };
}
pub(crate) use packet_enum;

macro_rules! def_user_enum {
    (
        $enum_ident:ident ($enum_ty:ty) {
            $(
                $variant_ident:ident = $variant_discriminator:expr
            ),*
        }
    ) => {

        #[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
        pub enum $enum_ident {
            $(
                $variant_ident
            ),*
        }

        impl crate::io::Readable for $enum_ident {
            fn read_from(data: &mut std::io::Cursor<&[u8]>) -> std::result::Result<Self, anyhow::Error> {
                match <$enum_ty>::read_from(data)?.saturating_add(0) { // deref workaround
                    $(
                        $variant_discriminator => Ok(Self::$variant_ident),
                    )*
                    n => anyhow::bail!("unknown variant {n} for {:?}!", std::any::type_name::<Self>())
                }
            }
        }
        
        impl crate::io::Writable for $enum_ident {
            fn write_to(&self, target: &mut std::vec::Vec<u8>) -> std::result::Result<(), anyhow::Error> {
                match self {
                    $(
                        Self::$variant_ident => <$enum_ty>::from($variant_discriminator).write_to(target)?
                    ),*
                };
                Ok(())
            }
        }
    };
}
pub(crate) use def_user_enum;