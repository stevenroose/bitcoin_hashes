// Bitcoin Hashes Library
// Written in 2019 by
//   The rust-bitcoin developers.
//
// To the extent possible under law, the author(s) have dedicated all
// copyright and related and neighboring rights to this software to
// the public domain worldwide. This software is distributed without
// any warranty.
//
// You should have received a copy of the CC0 Public Domain Dedication
// along with this software.
// If not, see <http://creativecommons.org/publicdomain/zero/1.0/>.
//

//! # SHA256t (tagged SHA256)

use core::marker::PhantomData;

use sha256;
use Hash as HashTrait;
#[allow(unused)]
use Error;

/// Trait representing a tag that can be used as a context for SHA256t hashes.
pub trait Tag: Copy + Ord + Default + ::core::hash::Hash {
    /// Returns a hash engine that is pre-tagged and is ready
    /// to be used for the data.
    fn engine() -> sha256::HashEngine;
}

/// Output of the SHA256t hash function.
#[derive(Copy, Clone, PartialEq, Eq, Default, PartialOrd, Ord, Hash)]
pub struct Hash<T: Tag>([u8; 32], PhantomData<T>);

hex_fmt_impl!(Debug, Hash, T:Tag);
hex_fmt_impl!(Display, Hash, T:Tag);
hex_fmt_impl!(LowerHex, Hash, T:Tag);
index_impl!(Hash, T:Tag);
borrow_slice_impl!(Hash, T:Tag);

impl<T: Tag> HashTrait for Hash<T> {
    type Engine = sha256::HashEngine;
    type Inner = [u8; 32];

    fn engine() -> sha256::HashEngine {
        T::engine()
    }

    fn from_engine(e: sha256::HashEngine) -> Hash<T> {
        Hash::from_inner(sha256::Hash::from_engine(e).into_inner())
    }

    const LEN: usize = 32;

    fn from_slice(sl: &[u8]) -> Result<Hash<T>, Error> {
        if sl.len() != 32 {
            Err(Error::InvalidLength(Self::LEN, sl.len()))
        } else {
            let mut ret = [0; 32];
            ret.copy_from_slice(sl);
            Ok(Hash::from_inner(ret))
        }
    }

    // NOTE! If this is changed, please make sure the serde serialization is still correct.
    const DISPLAY_BACKWARD: bool = true;

    fn into_inner(self) -> Self::Inner {
        self.0
    }

    fn from_inner(inner: Self::Inner) -> Self {
        Hash(inner, PhantomData)
    }
}

#[cfg(feature="serde")]
impl<T: Tag> ::serde::Serialize for Hash<T> {
    fn serialize<S: ::serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        use ::hex::ToHex;
        if s.is_human_readable() {
            s.serialize_str(&self.to_hex())
        } else {
            s.serialize_bytes(&self[..])
        }
    }
}

#[cfg(feature="serde")]
#[derive(Default)]
struct HexVisitor<T: Tag>(PhantomData<T>);

impl<'de, T: Tag> ::serde::de::Visitor<'de> for HexVisitor<T> {
    type Value = Hash<T>;

    fn expecting(&self, formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        formatter.write_str("an ASCII hex string")
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
        where
            E: ::serde::de::Error,
    {
        use ::hex::FromHex;
        if let Ok(hex) = ::std::str::from_utf8(v) {
            Hash::<T>::from_hex(hex).map_err(E::custom)
        } else {
            return Err(E::invalid_value(::serde::de::Unexpected::Bytes(v), &self));
        }
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: ::serde::de::Error,
    {
        use ::hex::FromHex;
        Hash::<T>::from_hex(v).map_err(E::custom)
    }
}

#[cfg(feature="serde")]
#[derive(Default)]
struct BytesVisitor<T: Tag>(PhantomData<T>);

impl<'de, T: Tag> ::serde::de::Visitor<'de> for BytesVisitor<T> {
    type Value = Hash<T>;

    fn expecting(&self, formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        formatter.write_str("a bytestring")
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
        where
            E: ::serde::de::Error,
    {
        Hash::<T>::from_slice(v).map_err(|_| {
            // from_slice only errors on incorrect length
            E::invalid_length(v.len(), &"32")
        })
    }
}

#[cfg(feature="serde")]
impl<'de, T: Tag> ::serde::Deserialize<'de> for Hash<T> {
    fn deserialize<D: ::serde::Deserializer<'de>>(d: D) -> Result<Hash<T>, D::Error> {
        if d.is_human_readable() {
            d.deserialize_str(HexVisitor::<T>::default())
        } else {
            d.deserialize_bytes(BytesVisitor::<T>::default())
        }
    }
}
