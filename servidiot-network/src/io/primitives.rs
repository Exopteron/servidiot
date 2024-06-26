use anyhow::bail;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use nbt::{from_gzip_reader, to_gzip_writer};
use servidiot_primitives::{
    item::{InventorySlot, ItemStack},
    metadata::{Metadata, MetadataItem, MetadataTypeKey},
    number::{FixedPoint, RotationFraction360},
    player::Gamemode,
    position::BlockPosition,
};

use super::{Readable, Serializable, Writable};

use std::{
    error::Error,
    io::{Cursor, Read, Write},
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

macro_rules! integer_impl {
    (
        $({$int:ty, $read:ident, $write:ident}),*
    ) => {
        $(
            impl Readable for $int {
                fn read_from(data: &mut Cursor<&[u8]>) -> anyhow::Result<Self> {
                    Ok(data.$read::<byteorder::BigEndian>()?)
                }
            }

            impl Writable for $int {
                fn write_to(&self, target: &mut Vec<u8>) -> anyhow::Result<()> {
                    target.$write::<byteorder::BigEndian>(*self)?;
                    Ok(())
                }
            }
        )*
    };
}

integer_impl!(
    {i16, read_i16, write_i16},
    {i32, read_i32, write_i32},
    {i64, read_i64, write_i64},
    {u16, read_u16, write_u16},
    {u32, read_u32, write_u32},
    {u64, read_u64, write_u64}
);

impl Readable for i8 {
    fn read_from(data: &mut Cursor<&[u8]>) -> anyhow::Result<Self> {
        Ok(data.read_i8()?)
    }
}
impl Readable for u8 {
    fn read_from(data: &mut Cursor<&[u8]>) -> anyhow::Result<Self> {
        Ok(data.read_u8()?)
    }
}
impl Writable for i8 {
    fn write_to(&self, target: &mut Vec<u8>) -> anyhow::Result<()> {
        target.write_i8(*self)?;
        Ok(())
    }
}
impl Writable for u8 {
    fn write_to(&self, target: &mut Vec<u8>) -> anyhow::Result<()> {
        target.write_u8(*self)?;
        Ok(())
    }
}

impl Readable for f32 {
    fn read_from(data: &mut Cursor<&[u8]>) -> anyhow::Result<Self> {
        Ok(data.read_f32::<BigEndian>()?)
    }
}
impl Readable for f64 {
    fn read_from(data: &mut Cursor<&[u8]>) -> anyhow::Result<Self> {
        Ok(data.read_f64::<BigEndian>()?)
    }
}
impl Writable for f32 {
    fn write_to(&self, target: &mut Vec<u8>) -> anyhow::Result<()> {
        target.write_f32::<BigEndian>(*self)?;
        Ok(())
    }
}
impl Writable for f64 {
    fn write_to(&self, target: &mut Vec<u8>) -> anyhow::Result<()> {
        target.write_f64::<BigEndian>(*self)?;
        Ok(())
    }
}

impl Readable for bool {
    fn read_from(data: &mut Cursor<&[u8]>) -> anyhow::Result<Self> {
        Ok(data.read_u8()? != 0)
    }
}
impl Writable for bool {
    fn write_to(&self, target: &mut Vec<u8>) -> anyhow::Result<()> {
        target.write_u8(*self as u8)?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct VarInt(pub i32);
impl VarInt {
    const SEGMENT_BITS: i32 = 0x7F;
    const CONTINUE_BIT: i32 = 0x80;
    const MAX_LEN: i32 = 32;
}
impl Deref for VarInt {
    type Target = i32;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Readable for VarInt {
    fn read_from(data: &mut Cursor<&[u8]>) -> anyhow::Result<Self> {
        let mut value = 0;
        let mut position = 0;
        loop {
            let current_byte = u8::read_from(data)? as i32;
            value |= (current_byte & Self::SEGMENT_BITS) << position;
            if (current_byte & Self::CONTINUE_BIT) == 0 {
                return Ok(Self(value));
            }
            position += 7;
            if position >= Self::MAX_LEN {
                bail!("VarInt too large!");
            }
        }
    }
}
impl Writable for VarInt {
    fn write_to(&self, target: &mut Vec<u8>) -> anyhow::Result<()> {
        let mut value = self.0;
        loop {
            if (value & !Self::SEGMENT_BITS) == 0 {
                (value as u8).write_to(target)?;
                return Ok(());
            }

            (((value & Self::SEGMENT_BITS) | Self::CONTINUE_BIT) as u8).write_to(target)?;

            value = ((value as u32) >> 7) as i32;
        }
    }
}
impl From<i32> for VarInt {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

impl TryFrom<usize> for VarInt {
    type Error = <i32 as TryFrom<usize>>::Error;
    fn try_from(value: usize) -> Result<Self, Self::Error> {
        Ok(Self(i32::try_from(value)?))
    }
}
impl TryInto<usize> for VarInt {
    type Error = <i32 as TryInto<usize>>::Error;

    fn try_into(self) -> Result<usize, Self::Error> {
        self.0.try_into()
    }
}

#[cfg(test)]
#[test]
fn varint_test() {
    fn varint(n: i32) -> Vec<u8> {
        let mut v = vec![];
        VarInt(n).write_to(&mut v).unwrap();
        v
    }

    macro_rules! test {
        ($a:expr, $b:expr) => {{
            assert_eq!(varint($a), $b);
            let n = $b.to_vec();
            let mut c = Cursor::new(n.as_slice());
            let v = VarInt::read_from(&mut c).unwrap();
            assert_eq!(v.0, $a);
        }};
    }

    let data = [42, 6, 64, 36, 0, 0];
    {
        let n = data.to_vec();
        let mut c = Cursor::new(n.as_slice());
        let v = VarInt::read_from(&mut c).unwrap();
        assert_eq!(v.0, 42);
    }
    test!(0, [0x00]);
    test!(1, [0x01]);
    test!(2, [0x02]);
    test!(127, [0x7f]);
    test!(128, [0x80, 0x01]);
    test!(255, [0xff, 0x01]);
    test!(25565, [0xdd, 0xc7, 0x01]);
    test!(2097151, [0xff, 0xff, 0x7f]);
    test!(2147483647, [0xff, 0xff, 0xff, 0xff, 0x07]);
    test!(-1, [0xff, 0xff, 0xff, 0xff, 0x0f]);
    test!(-2147483648, [0x80, 0x80, 0x80, 0x80, 0x08]);
}

#[derive(Debug)]
pub struct LengthPrefixedVec<L: Serializable, T: Serializable>(pub Vec<T>, PhantomData<L>);

impl<L: Serializable, T: Serializable> Deref for LengthPrefixedVec<L, T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<L: Serializable, T: Serializable> DerefMut for LengthPrefixedVec<L, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<L: Serializable, T: Serializable> LengthPrefixedVec<L, T> {
    pub fn new(vec: Vec<T>) -> Self {
        Self(vec, PhantomData)
    }
}

impl<L: TryInto<usize> + Serializable, T: Serializable> Readable for LengthPrefixedVec<L, T>
where
    <L as TryInto<usize>>::Error: Send + Sync + Error + 'static,
{
    fn read_from(data: &mut Cursor<&[u8]>) -> anyhow::Result<Self> {
        let len = L::read_from(data)?.try_into()?;
        let mut v = Vec::with_capacity(len);
        for _ in 0..len {
            v.push(T::read_from(data)?);
        }
        Ok(Self::new(v))
    }
}

impl<L: TryFrom<usize> + Serializable, T: Serializable> Writable for LengthPrefixedVec<L, T>
where
    <L as TryFrom<usize>>::Error: Send + Sync + Error + 'static,
{
    fn write_to(&self, target: &mut Vec<u8>) -> anyhow::Result<()> {
        let len = L::try_from(self.len())?;
        len.write_to(target)?;
        for element in &self.0 {
            element.write_to(target)?;
        }
        Ok(())
    }
}

pub type VarIntPrefixedByteArray = LengthPrefixedVec<VarInt, u8>;

impl Readable for String {
    fn read_from(data: &mut Cursor<&[u8]>) -> anyhow::Result<Self> {
        let data = VarIntPrefixedByteArray::read_from(data)?;
        Ok(Self::from_utf8(data.0)?)
    }
}
impl Writable for String {
    fn write_to(&self, target: &mut Vec<u8>) -> anyhow::Result<()> {
        VarIntPrefixedByteArray::new(self.as_bytes().to_vec()).write_to(target)
    }
}

impl Readable for Gamemode {
    fn read_from(data: &mut Cursor<&[u8]>) -> anyhow::Result<Self> {
        let n = u8::read_from(data)?;
        if let Some(v) = Self::decode(n) {
            Ok(v)
        } else {
            bail!("bad gamemode")
        }
    }
}
impl Writable for Gamemode {
    fn write_to(&self, target: &mut Vec<u8>) -> anyhow::Result<()> {
        self.encode().write_to(target)
    }
}

impl Readable for InventorySlot {
    fn read_from(data: &mut Cursor<&[u8]>) -> anyhow::Result<Self> {
        let id = i16::read_from(data)?;
        if id == -1 {
            return Ok(Self::Empty);
        }
        let item_count = i8::read_from(data)?;
        let item_meta = i16::read_from(data)?;
        let nbt_len = i16::read_from(data)?;
        let nbt = if nbt_len != -1 {
            let mut nbt = vec![0; nbt_len as usize];
            data.read_exact(&mut nbt)?;
            Some(from_gzip_reader(&mut Cursor::new(nbt.as_slice()))?)
        } else {
            None
        };
        Ok(Self::Filled(ItemStack {
            count: item_count,
            meta: item_meta,
            id,
            nbt_data: nbt,
        }))
    }
}

impl Writable for InventorySlot {
    fn write_to(&self, target: &mut Vec<u8>) -> anyhow::Result<()> {
        match self {
            InventorySlot::Empty => (-1i16).write_to(target),
            InventorySlot::Filled(stack) => {
                stack.id.write_to(target)?;
                stack.count.write_to(target)?;
                stack.meta.write_to(target)?;
                match &stack.nbt_data {
                    Some(data) => {
                        let mut out = vec![];
                        to_gzip_writer(&mut out, &data, None)?;
                        let len: i16 = out.len().try_into()?;
                        len.write_to(target)?;
                        target.append(&mut out);
                        Ok(())
                    }
                    None => (-1i16).write_to(target),
                }
            }
        }
    }
}

impl Writable for RotationFraction360 {
    fn write_to(&self, buffer: &mut Vec<u8>) -> anyhow::Result<()> {
        let num = ff((self.0 * 256.0) / 360.);
        if (num as i8) < 0 {
            //log::info!("Num {} Casted {} Modulo {}", num, num as i8, num % i8::MAX as i32);
        }
        //num = num.min(0);
        (num as u8).write_to(buffer)?;
        Ok(())
    }
}
fn ff(input: f32) -> i32 {
    let v = input as i32;
    if input < v as f32 {
        v - 1
    } else {
        v
    }
}
impl Readable for RotationFraction360 {
    fn read_from(data: &mut Cursor<&[u8]>) -> anyhow::Result<Self> {
        let num = i8::read_from(data)? as i32;
        let float = ((num * 360) as f32) / 256.;
        Ok(Self(float))
    }
}

impl Writable for FixedPoint {
    fn write_to(&self, target: &mut Vec<u8>) -> anyhow::Result<()> {
        target.write_all(&self.to_be_bytes())?;
        Ok(())
    }
}

impl Readable for FixedPoint {
    fn read_from(data: &mut Cursor<&[u8]>) -> anyhow::Result<Self> {
        let mut read = [0; 4];
        data.read_exact(&mut read)?;
        Ok(Self::from_be_bytes(read))
    }
}

impl Writable for Metadata {
    fn write_to(&self, target: &mut Vec<u8>) -> anyhow::Result<()> {
        for (key, item) in self.values() {
            target.write_all(&[key | ((item.type_key() as u8) << 5)])?;

            match item {
                MetadataItem::Byte(v) => v.write_to(target)?,
                MetadataItem::Short(v) => v.write_to(target)?,
                MetadataItem::Int(v) => v.write_to(target)?,
                MetadataItem::Float(v) => v.write_to(target)?,
                MetadataItem::String(v) => v.write_to(target)?,
                MetadataItem::Slot => todo!(),
                MetadataItem::Position(v) => {
                    v.x.write_to(target)?;
                    v.y.write_to(target)?;
                    v.z.write_to(target)?;
                }
            }
        }
        target.write_all(&[127])?; // end marker
        Ok(())
    }
}

impl Readable for Metadata {
    fn read_from(data: &mut Cursor<&[u8]>) -> anyhow::Result<Self> {
        let mut values = Metadata::default();
        loop {
            let k = u8::read_from(data)?;
            if k == 127 {
                break;
            }

            let key_value = k & 0x1F;

            match k >> 5 {
                v if v == MetadataTypeKey::Byte as u8 => {
                    values.insert(key_value, MetadataItem::Byte(u8::read_from(data)?))
                }
                v if v == MetadataTypeKey::Short as u8 => {
                    values.insert(key_value, MetadataItem::Short(i16::read_from(data)?))
                }
                v if v == MetadataTypeKey::Int as u8 => {
                    values.insert(key_value, MetadataItem::Int(i32::read_from(data)?))
                }
                v if v == MetadataTypeKey::Float as u8 => {
                    values.insert(key_value, MetadataItem::Float(f32::read_from(data)?))
                }
                v if v == MetadataTypeKey::String as u8 => {
                    values.insert(key_value, MetadataItem::String(String::read_from(data)?))
                }
                v if v == MetadataTypeKey::Slot as u8 => todo!(),
                v if v == MetadataTypeKey::Position as u8 => values.insert(
                    key_value,
                    MetadataItem::Position(BlockPosition::new(
                        i32::read_from(data)?,
                        i32::read_from(data)?,
                        i32::read_from(data)?,
                    )),
                ),
                v => bail!("unknown metadata type {}", v),
            }
        }
        Ok(values)
    }
}
