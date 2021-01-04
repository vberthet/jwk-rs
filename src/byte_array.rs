use generic_array::{ArrayLength, GenericArray};
use std::fmt;

use derive_more::{AsRef, Deref, From};
use serde::{
    de::{self, Deserialize, Deserializer},
    ser::{Serialize, Serializer},
};
use zeroize::{Zeroize, Zeroizing};

use crate::utils::{deserialize_base64, serialize_base64};

/// A zeroizing-on-drop container for a `[u8; N]` that deserializes from base64.
#[derive(Clone, Zeroize, Deref, AsRef, From)]
pub struct ByteArray<N: ArrayLength<u8>>(pub GenericArray<u8, N>);

impl<N: ArrayLength<u8>> fmt::Debug for ByteArray<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if cfg!(debug_assertions) {
            write!(f, "{}", base64::encode(self.0.as_slice()))
        } else {
            write!(f, "ByteArray<{}>", N::to_usize())
        }
    }
}

impl<N: ArrayLength<u8>> PartialEq for ByteArray<N> {
    fn eq(&self, other: &Self) -> bool {
        self.0.as_slice() == other.0.as_slice()
    }
}

impl<N: ArrayLength<u8>> Eq for ByteArray<N> {}

impl<N: ArrayLength<u8>> ByteArray<N> {
    pub fn try_from_slice(bytes: impl AsRef<[u8]>) -> Result<Self, String> {
        let bytes = bytes.as_ref();
        if bytes.len() != N::to_usize() {
            Err(format!(
                "expected {} bytes but got {}",
                N::to_usize(),
                bytes.len()
            ))
        } else {
            Ok(ByteArray(GenericArray::clone_from_slice(bytes)))
        }
    }
}

impl<N: ArrayLength<u8>> Serialize for ByteArray<N> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        serialize_base64(self.0.as_slice(), s)
    }
}

impl<'de, N: ArrayLength<u8>> Deserialize<'de> for ByteArray<N> {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let bytes = Zeroizing::new(deserialize_base64(d)?);
        Self::try_from_slice(&*bytes).map_err(|_| {
            de::Error::invalid_length(
                bytes.len(),
                &format!("{} base64-encoded bytes", N::to_usize()).as_str(),
            )
        })
    }
}

#[cfg(test)]
mod tests {
    use generic_array::typenum::U0;
    use generic_array::typenum::U6;
    use generic_array::typenum::U7;
    use generic_array::typenum::U8;

    use super::*;

    static BYTES: &[u8] = &[1, 2, 3, 4, 5, 6, 7];
    static BASE64_JSON: &str = "\"AQIDBAUGBw==\"";

    fn get_de() -> serde_json::Deserializer<serde_json::de::StrRead<'static>> {
        serde_json::Deserializer::from_str(&BASE64_JSON)
    }

    #[test]
    fn test_serde_byte_array_good() {
        let arr = ByteArray::<U7>::try_from_slice(BYTES).unwrap();
        let b64 = serde_json::to_string(&arr).unwrap();
        assert_eq!(b64, BASE64_JSON);
        let bytes: ByteArray<U7> = serde_json::from_str(&b64).unwrap();
        assert_eq!(bytes.0.as_slice(), BYTES);
    }

    #[test]
    fn test_serde_deserialize_byte_array_invalid() {
        let mut de = serde_json::Deserializer::from_str("\"Z\"");
        ByteArray::<U0>::deserialize(&mut de).unwrap_err();
    }

    #[test]
    fn test_serde_base64_deserialize_array_long() {
        ByteArray::<U6>::deserialize(&mut get_de()).unwrap_err();
    }

    #[test]
    fn test_serde_base64_deserialize_array_short() {
        ByteArray::<U8>::deserialize(&mut get_de()).unwrap_err();
    }
}
